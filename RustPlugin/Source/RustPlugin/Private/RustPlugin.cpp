// Copyright Epic Games, Inc. All Rights Reserved.

#include "RustPlugin.h"
#include "RustPluginStyle.h"
#include "RustPluginCommands.h"
#include "FRustDetailCustomization.h"
#include "LevelEditor.h"
#include "Widgets/Docking/SDockTab.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Text/STextBlock.h"
#include "ToolMenus.h"
#include "DirectoryWatcherModule.h"
#include "IDirectoryWatcher.h"
#include "Misc/Paths.h"
#include "FUuidGraphPanelPinFactory.h"
#include "Modules/ModuleManager.h"
#include "RustUtils.h"
#include "Framework/Notifications/NotificationManager.h"
#include "Widgets/Notifications/SNotificationList.h"

static const FName RustPluginTabName("RustPlugin");

#define LOCTEXT_NAMESPACE "FRustPluginModule"

FString PlatformExtensionName()
{
#if PLATFORM_LINUX || PLATFORM_MAC
	return FString(TEXT("so"));
#elif PLATFORM_WINDOWS
        return FString(TEXT("dll"));
#endif
}

FString FPlugin::PluginFolderPath()
{
	return FPaths::Combine(*FPaths::ConvertRelativePathToFull(FPaths::ProjectDir()), TEXT("Binaries"));
}

FString FPlugin::PluginPath()
{
	return FPaths::Combine(*PluginFolderPath(), *PluginFileName());
}

FString FPlugin::PluginFileName()
{
	return FString::Printf(TEXT("%s.%s"), TEXT("rustplugin"), *PlatformExtensionName());
}

FPlugin::FPlugin()
{
}

bool FPlugin::TryLoad()
{
	FString Path = PluginPath();
	FString LocalTargetPath = FPaths::Combine(*PluginFolderPath(),
	                                          *FString::Printf(
		                                          TEXT("%s-%i"), *PluginFileName(),
		                                          FDateTime::Now().ToUnixTimestamp()));
	if (this->IsLoaded())
	{
		FPlatformProcess::FreeDllHandle(this->Handle);
		this->Handle = nullptr;
		// This is leaky. If we close the editor this will not delete the file
		if (!FPlatformFileManager::Get().GetPlatformFile().DeleteFile(*this->TargetPath))
		{
			UE_LOG(LogTemp, Warning, TEXT("Unable to delete File %s"), *this->TargetPath);
		}
	}

	//UE_LOG(LogTemp, Warning, TEXT("Copy From %s to %s"), *Path, *LocalTargetPath);
	if (!FPlatformFileManager::Get().GetPlatformFile().CopyFile(*LocalTargetPath, *Path))
	{
		UE_LOG(LogTemp, Warning, TEXT("Unable to copy File from %s to %s"), *Path, *LocalTargetPath);
		return false;
	}
	void* LocalHandle = FPlatformProcess::GetDllHandle(*LocalTargetPath);

	if (LocalHandle == nullptr)
	{
		UE_LOG(LogTemp, Warning, TEXT("Dll open failed"));
		return false;
	}

	this->Handle = LocalHandle;

	void* LocalBindings = FPlatformProcess::GetDllExport(LocalHandle, TEXT("register_unreal_bindings\0"));
	ensure(LocalBindings);

	this->Bindings = (EntryUnrealBindingsFn)LocalBindings;

	this->TargetPath = LocalTargetPath;
	NeedsInit = true;
	CallEntryPoints();
	return true;
}

void FRustPluginModule::Exit()
{
	if (GEditor)
	{
		GEditor->RequestEndPlayMap();
	}
}

bool FPlugin::IsLoaded()
{
	return Handle != nullptr;
}

void FPlugin::CallEntryPoints()
{
	if (!IsLoaded())
		return;

	// Pass unreal function pointers to Rust, and also retrieve function pointers from Rust so we can call into Rust
	if (Bindings(CreateBindings(), &Rust))
	{
		RetrieveReflectionData();
	}
	else
	{
		// TODO: We had a panic when calling the entry point. We need to handle that better, otherwise unreal will segfault because the rust bindings are nullptrs
	}
}

void FPlugin::RetrieveReflectionData()
{
	uintptr_t len = 0;
	Rust.retrieve_uuids(nullptr, &len);
	TArray<Uuid> LocalUuids;
	LocalUuids.Reserve(len);
	Rust.retrieve_uuids(LocalUuids.GetData(), &len);
	LocalUuids.SetNum(len);

	ReflectionData.Types.Reset();

	for (Uuid Id : LocalUuids)
	{
		uint32_t NumberOfFields = 0;
		Rust.reflection_fns.number_of_fields(Id, &NumberOfFields);
		Utf8Str TypeNameStr;
		// TODO: Better error handling here. None of this should fail though
		check(Rust.reflection_fns.get_type_name(Id, &TypeNameStr));

		FRustReflection Reflection;
		Reflection.Name = ToFString(TypeNameStr);
		Reflection.IsEditorComponent = Rust.reflection_fns.is_editor_component(Id) == 1;
		
		for (uint32_t Idx = 0; Idx < NumberOfFields; Idx++)
		{
			Utf8Str FieldNamePtr;
			check(Rust.reflection_fns.get_field_name(Id, Idx, &FieldNamePtr));
			ReflectionType Type = ReflectionType::Bool;
			check(Rust.reflection_fns.get_field_type(Id, Idx, &Type));

			FString FieldName = ToFString(FieldNamePtr);
			Reflection.IndexToFieldName.Add(Idx, FieldName);
			Reflection.FieldNameToType.Add(FieldName, Type);
		}
		
		ReflectionData.Types.Add(ToFGuid(Id), Reflection);
	}
}

void FRustPluginModule::StartupModule()
{
	// This code will execute after your module is loaded into memory; the exact timing is specified in the .uplugin file per-module

	FRustPluginStyle::Initialize();
	FRustPluginStyle::ReloadTextures();

	FRustPluginCommands::Register();

	//PluginCommands = MakeShareable(new FUICommandList);

	//PluginCommands->MapAction(
	//    FRustPluginCommands::Get().OpenPluginWindow,
	//    FExecuteAction::CreateRaw(this, &FRustPluginModule::PluginButtonClicked),
	//    FCanExecuteAction());

	//UToolMenus::RegisterStartupCallback(FSimpleMulticastDelegate::FDelegate::CreateRaw(this, &FRustPluginModule::RegisterMenus));

	//FGlobalTabmanager::Get()->RegisterNomadTabSpawner(RustPluginTabName, FOnSpawnTab::CreateRaw(this, &FRustPluginModule::OnSpawnPluginTab)).SetDisplayName(LOCTEXT("FRustPluginTabTitle", "RustPlugin")).SetMenuType(ETabSpawnerMenuType::Hidden);

	IDirectoryWatcher* watcher = FModuleManager::LoadModuleChecked<FDirectoryWatcherModule>(
			TEXT("DirectoryWatcher")).
		Get();
	watcher->RegisterDirectoryChangedCallback_Handle(
		*Plugin.PluginFolderPath(),
		IDirectoryWatcher::FDirectoryChanged::CreateRaw(this, &FRustPluginModule::OnProjectDirectoryChanged),
		WatcherHandle, IDirectoryWatcher::WatchOptions::IgnoreChangesInSubtree);
	Plugin.TryLoad();

	TSharedPtr<FUuidGraphPanelPinFactory> UuidFactory = MakeShareable(new FUuidGraphPanelPinFactory());
	FEdGraphUtilities::RegisterVisualPinFactory(UuidFactory);

	// Register detail customizations
	{
		auto& PropertyModule = FModuleManager::LoadModuleChecked<FPropertyEditorModule>("PropertyEditor");

		PropertyModule.RegisterCustomClassLayout(
			"EntityComponent",
			FOnGetDetailCustomizationInstance::CreateStatic(&FRustDetailCustomization::MakeInstance)
		);

		PropertyModule.NotifyCustomizationModuleChanged();
	}
}

void FRustPluginModule::OnProjectDirectoryChanged(const TArray<FFileChangeData>& Data)
{
	for (FFileChangeData Changed : Data)
	{
		FString Name = FPaths::GetBaseFilename(Changed.Filename);
		FString Ext = FPaths::GetExtension(Changed.Filename, false);

		FString Leaf = FPaths::GetPathLeaf(FPaths::GetPath(Changed.Filename));
		const bool ChangedOrAdded = Changed.Action == FFileChangeData::FCA_Added || Changed.Action ==
			FFileChangeData::FCA_Modified;
		if (Name == TEXT("rustplugin") && Ext == *PlatformExtensionName() && ChangedOrAdded)
		{
			Plugin.TryLoad();
			// Only show notifications when we are in playmode otherwise notifications are a bit too spamy
			if (GEditor != nullptr && GEditor->IsPlaySessionInProgress())
			{
				// Still too spamy

				//FNotificationInfo Info(LOCTEXT("SpawnNotification_Notification", "Hotreload: Rust"));
				//Info.ExpireDuration = 1.0f;
				//FSlateNotificationManager::Get().AddNotification(Info);
			}

			UE_LOG(LogTemp, Display, TEXT("Hotreload: Rust"));

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

TSharedRef<SDockTab> FRustPluginModule::OnSpawnPluginTab(const FSpawnTabArgs& SpawnTabArgs)
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
		UToolMenu* Menu = UToolMenus::Get()->ExtendMenu("LevelEditor.MainMenu.Window");
		{
			FToolMenuSection& Section = Menu->FindOrAddSection("WindowLayout");
			Section.AddMenuEntryWithCommandList(FRustPluginCommands::Get().OpenPluginWindow, PluginCommands);
		}
	}

	{
		UToolMenu* ToolbarMenu = UToolMenus::Get()->ExtendMenu("LevelEditor.LevelEditorToolBar");
		{
			FToolMenuSection& Section = ToolbarMenu->FindOrAddSection("Settings");
			{
				FToolMenuEntry& Entry = Section.AddEntry(
					FToolMenuEntry::InitToolBarButton(FRustPluginCommands::Get().OpenPluginWindow));
				Entry.SetCommandList(PluginCommands);
			}
		}
	}
}

#undef LOCTEXT_NAMESPACE

// IMPLEMENT_MODULE(FRustPluginModule, RustPlugin)
IMPLEMENT_PRIMARY_GAME_MODULE(FRustPluginModule, RustPlugin, "RustPlugin");
