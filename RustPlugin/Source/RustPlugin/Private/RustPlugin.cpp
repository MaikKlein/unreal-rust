// Copyright Epic Games, Inc. All Rights Reserved.

#include "RustPlugin.h"
#include "RustPluginStyle.h"
#include "RustPluginCommands.h"
#include "LevelEditor.h"
#include "Widgets/Docking/SDockTab.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Text/STextBlock.h"
#include "ToolMenus.h"
#include "Containers/Ticker.h"
#include "DirectoryWatcherModule.h"
#include "IDirectoryWatcher.h"
#include "HAL/PlatformFilemanager.h"
#include "Misc/Paths.h"
#include "Api.h"

static const FName RustPluginTabName("RustPlugin");

#define LOCTEXT_NAMESPACE "FRustPluginModule"

bool FPlugin::TryLoad(FString &Path)
{
	FString LocalTargetPath = FString::Printf(TEXT("%srusttemp/%s-%i"),
											  *FPaths::ConvertRelativePathToFull(FPaths::ProjectDir()),
											  TEXT("rustplugin.dll"), FDateTime::Now().ToUnixTimestamp());
	if (this->IsLoaded())
	{
		FPlatformProcess::FreeDllHandle(this->Handle);
		FPlatformFileManager::Get().GetPlatformFile().DeleteFile(*this->TargetPath);
	}

	UE_LOG(LogTemp, Warning, TEXT("Copy From %s to %s"), *Path, *LocalTargetPath);
	FPlatformFileManager::Get().GetPlatformFile().CopyFile(*LocalTargetPath, *Path);
	void *LocalHandle = FPlatformProcess::GetDllHandle(*LocalTargetPath);
	this->Handle = LocalHandle;
	if (this->Handle == nullptr)
		return false;

	void *LocalBindings = FPlatformProcess::GetDllExport(LocalHandle, TEXT("register_unreal_bindings\0"));
	void *LocalBeginPlay = FPlatformProcess::GetDllExport(LocalHandle, TEXT("begin_play\0"));

	this->Bindings = (EntryUnrealBindingsFn)LocalBindings;
	this->BeginPlay = (EntryBeginPlayFn)LocalBeginPlay;
	this->TargetPath = LocalTargetPath;
	CallEntryPoints();
	return true;
}

bool FPlugin::IsLoaded()
{
	return Handle != nullptr;
}
void FPlugin::CallEntryPoints()
{
	if (!IsLoaded())
		return;

	Bindings(CreateBindings());
	Log("Foo");
	BeginPlay();
}

bool FRustPluginModule::Tick(float dt)
{
	return true;
}

void FRustPluginModule::StartupModule()
{
	// This code will execute after your module is loaded into memory; the exact timing is specified in the .uplugin file per-module

	FRustPluginStyle::Initialize();
	FRustPluginStyle::ReloadTextures();

	FRustPluginCommands::Register();

	PluginCommands = MakeShareable(new FUICommandList);

	PluginCommands->MapAction(
		FRustPluginCommands::Get().OpenPluginWindow,
		FExecuteAction::CreateRaw(this, &FRustPluginModule::PluginButtonClicked),
		FCanExecuteAction());

	UToolMenus::RegisterStartupCallback(FSimpleMulticastDelegate::FDelegate::CreateRaw(this, &FRustPluginModule::RegisterMenus));

	FGlobalTabmanager::Get()->RegisterNomadTabSpawner(RustPluginTabName, FOnSpawnTab::CreateRaw(this, &FRustPluginModule::OnSpawnPluginTab)).SetDisplayName(LOCTEXT("FRustPluginTabTitle", "RustPlugin")).SetMenuType(ETabSpawnerMenuType::Hidden);

	TickDelegate = FTickerDelegate::CreateRaw(this, &FRustPluginModule::Tick);
	TickDelegateHandle = FTicker::GetCoreTicker().AddTicker(TickDelegate);

	IDirectoryWatcher *watcher = FModuleManager::LoadModuleChecked<FDirectoryWatcherModule>(TEXT("DirectoryWatcher")).Get();
	watcher->RegisterDirectoryChangedCallback_Handle(
		"F:/unreal/unreal/example/UnrealLearningKitGames/rusttemp",
		IDirectoryWatcher::FDirectoryChanged::CreateRaw(this, &FRustPluginModule::OnProjectDirectoryChanged), WatcherHandle, IDirectoryWatcher::WatchOptions::IgnoreChangesInSubtree);
	FString P = FString(TEXT("F:/unreal/unreal/example/UnrealLearningKitGames/rusttemp/unreal_rust_example.dll"));
	Plugin.TryLoad(P);
}
void FRustPluginModule::OnProjectDirectoryChanged(const TArray<FFileChangeData> &Data)
{
	for (FFileChangeData Changed : Data)
	{
		FString Name = FPaths::GetBaseFilename(Changed.Filename);
		FString Ext = FPaths::GetExtension(Changed.Filename, false);
		FString Leaf = FPaths::GetPathLeaf(FPaths::GetPath(Changed.Filename));
		if (Name == TEXT("unreal_rust_example") && Ext == TEXT("dll"))
		{
			Plugin.TryLoad(Changed.Filename);
			return;
		}
	}
}

void FRustPluginModule::ShutdownModule()
{
	// This function may be called during shutdown to clean up your module.  For modules that support dynamic reloading,
	// we call this function before unloading the module.

	UToolMenus::UnRegisterStartupCallback(this);

	UToolMenus::UnregisterOwner(this);

	FRustPluginStyle::Shutdown();

	FRustPluginCommands::Unregister();

	FGlobalTabmanager::Get()->UnregisterNomadTabSpawner(RustPluginTabName);
}

TSharedRef<SDockTab> FRustPluginModule::OnSpawnPluginTab(const FSpawnTabArgs &SpawnTabArgs)
{
	FText WidgetText = FText::Format(
		LOCTEXT("WindowWidgetText", "BAR Add code to {0} in {1} to override this window's contents"),
		FText::FromString(TEXT("FRustPluginModule::OnSpawnPluginTab")),
		FText::FromString(TEXT("RustPlugin.cpp")));

	return SNew(SDockTab)
		.TabRole(ETabRole::NomadTab)
			[
				// Put your tab content here!
				SNew(SBox)
					.HAlign(HAlign_Center)
					.VAlign(VAlign_Center)
						[SNew(STextBlock)
							 .Text(WidgetText)]];
}

void FRustPluginModule::PluginButtonClicked()
{
	FGlobalTabmanager::Get()->TryInvokeTab(RustPluginTabName);
}

void FRustPluginModule::RegisterMenus()
{
	// Owner will be used for cleanup in call to UToolMenus::UnregisterOwner
	FToolMenuOwnerScoped OwnerScoped(this);

	{
		UToolMenu *Menu = UToolMenus::Get()->ExtendMenu("LevelEditor.MainMenu.Window");
		{
			FToolMenuSection &Section = Menu->FindOrAddSection("WindowLayout");
			Section.AddMenuEntryWithCommandList(FRustPluginCommands::Get().OpenPluginWindow, PluginCommands);
		}
	}

	{
		UToolMenu *ToolbarMenu = UToolMenus::Get()->ExtendMenu("LevelEditor.LevelEditorToolBar");
		{
			FToolMenuSection &Section = ToolbarMenu->FindOrAddSection("Settings");
			{
				FToolMenuEntry &Entry = Section.AddEntry(FToolMenuEntry::InitToolBarButton(FRustPluginCommands::Get().OpenPluginWindow));
				Entry.SetCommandList(PluginCommands);
			}
		}
	}
}

#undef LOCTEXT_NAMESPACE

// IMPLEMENT_MODULE(FRustPluginModule, RustPlugin)
IMPLEMENT_PRIMARY_GAME_MODULE(FRustPluginModule, RustPlugin, "RustPlugin");