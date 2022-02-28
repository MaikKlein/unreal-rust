// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"
#include "Containers/Ticker.h"
#include "Bindings.h"
#include "Misc/Optional.h"
#include "Containers/Array.h"


class FToolBarBuilder;
class FMenuBuilder;
class FDelegateHandle;
struct FFileChangeData;
class FString;
class ARustGameModeBase;

struct FPlugin {
	FString TargetPath;
	void* Handle;
	EntryUnrealBindingsFn Bindings;
	RustBindings Rust;

	bool NeedsInit;
	bool IsLoaded();
	bool TryLoad();
	void CallEntryPoints();
	void RetrieveUuids();
	FString PluginPath();

	TArray<Uuid> Uuids;
	FPlugin();
};

FString PlatformExtensionName();


class FRustPluginModule : public IModuleInterface
{
public:

	/** IModuleInterface implementation */
	virtual void StartupModule() override;
	virtual void ShutdownModule() override;
	
	/** This function will be bound to Command (by default it will bring up plugin window) */
	void PluginButtonClicked();
	
	FPlugin Plugin;
	bool ShouldReloadPlugin;
	ARustGameModeBase* GameMode;
	void Exit();
private:

	void RegisterMenus();

	TSharedRef<class SDockTab> OnSpawnPluginTab(const class FSpawnTabArgs& SpawnTabArgs);
	void OnProjectDirectoryChanged(const TArray<FFileChangeData> & Data);

private:
	TSharedPtr<class FUICommandList> PluginCommands;
	FDelegateHandle WatcherHandle;
};
