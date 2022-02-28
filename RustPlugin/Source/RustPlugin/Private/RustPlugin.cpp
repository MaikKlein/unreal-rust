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
#include "Misc/Paths.h"
#include "Api.h"
#include "Modules/ModuleManager.h"
#include "RustBindingsActor.h"
#include "Framework/Notifications/NotificationManager.h"
#include "Widgets/Notifications/SNotificationList.h"


static const FName RustPluginTabName("RustPlugin");

#define LOCTEXT_NAMESPACE "FRustPluginModule"

FString PlatformExtensionName() {
    #if PLATFORM_LINUX || PLATFORM_MAXOSX
        return FString(TEXT("so"));
    #elif PLATFORM_WINDOWS
        return FString(TEXT("dll"));
    #endif
}
FString FPlugin::PluginPath()
{
    return FPaths::Combine(*FPaths::ConvertRelativePathToFull(FPaths::ProjectDir()), TEXT("rusttemp"));
}
FPlugin::FPlugin()
{
}
bool FPlugin::TryLoad()
{
    UE_LOG(LogTemp, Warning, TEXT("TRY RELOAD"));
    FString Path = FPaths::Combine(*PluginPath(), TEXT("rustplugin.so"));
    FString LocalTargetPath = FPaths::Combine(*PluginPath(), *FString::Printf(TEXT("%s-%i"), TEXT("rustplugin.so"), FDateTime::Now().ToUnixTimestamp()));
    if (this->IsLoaded())
    {
        FPlatformProcess::FreeDllHandle(this->Handle);
        FPlatformFileManager::Get().GetPlatformFile().DeleteFile(*this->TargetPath);
    }

    UE_LOG(LogTemp, Warning, TEXT("Copy From %s to %s"), *Path, *LocalTargetPath);
    FPlatformFileManager::Get().GetPlatformFile().CopyFile(*LocalTargetPath, *Path);
    void *LocalHandle = FPlatformProcess::GetDllHandle(*LocalTargetPath);
    ensure(LocalHandle);
    this->Handle = LocalHandle;
    if (this->Handle == nullptr)
        return false;

    void *LocalBindings = FPlatformProcess::GetDllExport(LocalHandle, TEXT("register_unreal_bindings\0"));
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

    Rust = Bindings(CreateBindings());
    RetrieveUuids();
}
void FPlugin::RetrieveUuids()
{
    uintptr_t len = 0;
    Rust.retrieve_uuids(nullptr, &len);
    TArray<Uuid> LocalUuids;
    LocalUuids.Reserve(len);
    Rust.retrieve_uuids(LocalUuids.GetData(), &len);
    LocalUuids.SetNum(len);
    Uuids = LocalUuids;
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

    IDirectoryWatcher *watcher = FModuleManager::LoadModuleChecked<FDirectoryWatcherModule>(TEXT("DirectoryWatcher")).Get();
    watcher->RegisterDirectoryChangedCallback_Handle(
        *Plugin.PluginPath(),
        IDirectoryWatcher::FDirectoryChanged::CreateRaw(this, &FRustPluginModule::OnProjectDirectoryChanged), WatcherHandle, IDirectoryWatcher::WatchOptions::IgnoreChangesInSubtree);
    Plugin.TryLoad();
}
void FRustPluginModule::OnProjectDirectoryChanged(const TArray<FFileChangeData> &Data)
{
    for (FFileChangeData Changed : Data)
    {
        FString Name = FPaths::GetBaseFilename(Changed.Filename);
        FString Ext = FPaths::GetExtension(Changed.Filename, false);

        FString Leaf = FPaths::GetPathLeaf(FPaths::GetPath(Changed.Filename));
        if (Name == TEXT("rustplugin") && Ext == *PlatformExtensionName())
        {
            UE_LOG(LogTemp, Warning, TEXT("SHOULD RELOAD"));

            ShouldReloadPlugin = true;
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
