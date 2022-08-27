// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Bindings.h"
#include "Containers/Array.h"


class FToolBarBuilder;
class FMenuBuilder;
class FDelegateHandle;
struct FFileChangeData;
class FString;
class ARustGameModeBase;

struct FRustReflection
{
	FString Name;
	uint32 NumberOfFields;
	TMap<uint32, FString> IndexToFieldName;
	TMap<FString, ReflectionType> FieldNameToType;
};

struct FReflectionData
{
	TMap<FGuid, FRustReflection> Types;
};

struct FPlugin {
	FString TargetPath;
	void* Handle;
	EntryUnrealBindingsFn Bindings;
	RustBindings Rust;

	bool NeedsInit;
	bool IsLoaded();
	bool TryLoad();
	void CallEntryPoints();
	void RetrieveReflectionData();
	FString PluginFolderPath();
	FString PluginPath();
	FString PluginFileName();

	FReflectionData ReflectionData;
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
