// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"
#include "Containers/Ticker.h"
#include "Bindings.h"
#include "Misc/Optional.h"


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
	EntryBeginPlayFn BeginPlay;
	EntryTickFn Tick;


	bool NeedsInit;
	bool IsLoaded();
	bool TryLoad(FString& Path);
	void CallEntryPoints();
};


class FRustPluginModule : public IModuleInterface
{
public:

	/** IModuleInterface implementation */
	virtual void StartupModule() override;
	virtual void ShutdownModule() override;
	
	/** This function will be bound to Command (by default it will bring up plugin window) */
	void PluginButtonClicked();
	bool Tick(float dt);
	
	FPlugin Plugin;
	bool ShouldReloadPlugin;
	ARustGameModeBase* GameMode;
private:

	FTickerDelegate TickDelegate;
 	FDelegateHandle TickDelegateHandle;
	void RegisterMenus();

	TSharedRef<class SDockTab> OnSpawnPluginTab(const class FSpawnTabArgs& SpawnTabArgs);
	void OnProjectDirectoryChanged(const TArray<FFileChangeData> & Data);

private:
	TSharedPtr<class FUICommandList> PluginCommands;
	FDelegateHandle WatcherHandle;
};
