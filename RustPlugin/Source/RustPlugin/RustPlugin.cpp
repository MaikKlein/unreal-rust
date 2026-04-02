// Copyright Epic Games, Inc. All Rights Reserved.

#include "RustPlugin.h"
#include "RustPluginStyle.h"
#include "RustPluginCommands.h"
#include "LevelEditor.h"
#include "Widgets/Docking/SDockTab.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Text/STextBlock.h"
#include "ToolMenus.h"
#include "DirectoryWatcherModule.h"
#include "IDirectoryWatcher.h"
#include "Misc/Paths.h"
#include "Modules/ModuleManager.h"
#include "RustUtils.h"
#include "Bindings.h"
#include "HAL/PlatformFileManager.h"
#include "UObject/EnumProperty.h"
#include "UObject/UnrealType.h"
#include "RustClassDef.h"

#include <stdint.h>

#include "Kismet2/ReloadUtilities.h"

static const FName RustPluginTabName("RustPlugin");

#define LOCTEXT_NAMESPACE "FRustPluginModule"

namespace
{
	FProperty* CreatePropertyFromRustType(
		const FFieldVariant Owner,
		const FName& PropertyName,
		const EObjectFlags ObjectFlags,
		URustType* RustType)
	{
		check(RustType);

		if (Cast<URustType_Bool>(RustType) != nullptr)
		{
			return new FBoolProperty(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_Int8>(RustType) != nullptr)
		{
			return new FInt8Property(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_Int16>(RustType) != nullptr)
		{
			return new FInt16Property(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_Int32>(RustType) != nullptr)
		{
			return new FIntProperty(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_Int64>(RustType) != nullptr)
		{
			return new FInt64Property(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_UInt8>(RustType) != nullptr)
		{
			return new FByteProperty(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_UInt16>(RustType) != nullptr)
		{
			return new FUInt16Property(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_UInt32>(RustType) != nullptr)
		{
			return new FUInt32Property(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_UInt64>(RustType) != nullptr)
		{
			return new FUInt64Property(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_Float>(RustType) != nullptr)
		{
			return new FFloatProperty(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_Double>(RustType) != nullptr)
		{
			return new FDoubleProperty(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_FName>(RustType) != nullptr)
		{
			return new FNameProperty(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_FString>(RustType) != nullptr)
		{
			return new FStrProperty(Owner, PropertyName, ObjectFlags);
		}

		if (Cast<URustType_FText>(RustType) != nullptr)
		{
			return new FTextProperty(Owner, PropertyName, ObjectFlags);
		}

		if (URustType_UObject* ObjectType = Cast<URustType_UObject>(RustType))
		{
			FObjectProperty* ObjectProperty = new FObjectProperty(Owner, PropertyName, ObjectFlags);
			check(ObjectType->PropertyClass);
			ObjectProperty->PropertyClass = ObjectType->PropertyClass.Get();
			return ObjectProperty;
		}

		if (URustType_UClass* ClassType = Cast<URustType_UClass>(RustType))
		{
			FClassProperty* ClassProperty = new FClassProperty(Owner, PropertyName, ObjectFlags);
			ClassProperty->PropertyClass = UClass::StaticClass();
			check(ClassType->MetaClass);
			ClassProperty->SetMetaClass(ClassType->MetaClass.Get());
			return ClassProperty;
		}

		if (URustType_SoftObject* SoftObjectType = Cast<URustType_SoftObject>(RustType))
		{
			FSoftObjectProperty* SoftObjectProperty = new FSoftObjectProperty(Owner, PropertyName, ObjectFlags);
			check(SoftObjectType->PropertyClass);
			SoftObjectProperty->PropertyClass = SoftObjectType->PropertyClass.Get();
			return SoftObjectProperty;
		}

		if (URustType_SoftClass* SoftClassType = Cast<URustType_SoftClass>(RustType))
		{
			FSoftClassProperty* SoftClassProperty = new FSoftClassProperty(Owner, PropertyName, ObjectFlags);
			SoftClassProperty->PropertyClass = UClass::StaticClass();
			check(SoftClassType->MetaClass);
			SoftClassProperty->SetMetaClass(SoftClassType->MetaClass.Get());
			return SoftClassProperty;
		}

		if (URustType_WeakObject* WeakObjectType = Cast<URustType_WeakObject>(RustType))
		{
			FWeakObjectProperty* WeakObjectProperty = new FWeakObjectProperty(Owner, PropertyName, ObjectFlags);
			check(WeakObjectType->PropertyClass);
			WeakObjectProperty->PropertyClass = WeakObjectType->PropertyClass.Get();
			return WeakObjectProperty;
		}

		if (URustType_LazyObject* LazyObjectType = Cast<URustType_LazyObject>(RustType))
		{
			FLazyObjectProperty* LazyObjectProperty = new FLazyObjectProperty(Owner, PropertyName, ObjectFlags);
			check(LazyObjectType->PropertyClass);
			LazyObjectProperty->PropertyClass = LazyObjectType->PropertyClass.Get();
			return LazyObjectProperty;
		}

		if (URustType_Struct* StructType = Cast<URustType_Struct>(RustType))
		{
			check(StructType->ScriptStruct);
			FStructProperty* StructProperty = new FStructProperty(Owner, PropertyName, ObjectFlags);
			StructProperty->Struct = StructType->ScriptStruct.Get();
			return StructProperty;
		}

		if (URustType_Enum* EnumType = Cast<URustType_Enum>(RustType))
		{
			check(EnumType->Enum);
			FEnumProperty* EnumProperty = new FEnumProperty(Owner, PropertyName, ObjectFlags);
			FByteProperty* UnderlyingProperty = new FByteProperty(EnumProperty, TEXT("UnderlyingType"), ObjectFlags);
			EnumProperty->AddCppProperty(UnderlyingProperty);
			EnumProperty->SetEnum(EnumType->Enum.Get());
			return EnumProperty;
		}

		if (URustType_Array* ArrayType = Cast<URustType_Array>(RustType))
		{
			check(ArrayType->InnerType);
			FArrayProperty* ArrayProperty = new FArrayProperty(Owner, PropertyName, ObjectFlags);
			FProperty* InnerProperty = CreatePropertyFromRustType(
				ArrayProperty,
				TEXT("Inner"),
				ObjectFlags,
				ArrayType->InnerType.Get());

			ArrayProperty->AddCppProperty(InnerProperty);
			return ArrayProperty;
		}

		if (URustType_Set* SetType = Cast<URustType_Set>(RustType))
		{
			check(SetType->ElementType);
			FSetProperty* SetProperty = new FSetProperty(Owner, PropertyName, ObjectFlags);
			FProperty* ElementProperty = CreatePropertyFromRustType(
				SetProperty,
				TEXT("Element"),
				ObjectFlags,
				SetType->ElementType.Get());

			SetProperty->AddCppProperty(ElementProperty);
			return SetProperty;
		}

		if (URustType_Map* MapType = Cast<URustType_Map>(RustType))
		{
			check(MapType->KeyType);
			check(MapType->ValueType);
			FMapProperty* MapProperty = new FMapProperty(Owner, PropertyName, ObjectFlags);
			FProperty* KeyProperty = CreatePropertyFromRustType(
				MapProperty,
				TEXT("Key"),
				ObjectFlags,
				MapType->KeyType.Get());
			FProperty* ValueProperty = CreatePropertyFromRustType(
				MapProperty,
				TEXT("Value"),
				ObjectFlags,
				MapType->ValueType.Get());

			MapProperty->AddCppProperty(KeyProperty);
			MapProperty->AddCppProperty(ValueProperty);
			return MapProperty;
		}

		checkf(false, TEXT("Unsupported Rust type '%s'"), *RustType->GetClass()->GetName());
		return nullptr;
	}
}

FString PlatformExtensionName()
{
#if PLATFORM_MAC
	return FString(TEXT("dylib"));
#elif PLATFORM_LINUX
	return FString(TEXT("so"));
#elif PLATFORM_WINDOWS
	return FString(TEXT("dll"));
#endif
}

FString FRustLoader::PluginFolderPath()
{
	return FPaths::Combine(*FPaths::ConvertRelativePathToFull(FPaths::ProjectDir()), TEXT("Binaries"));
}

FString FRustLoader::PluginPath()
{
	return FPaths::Combine(*PluginFolderPath(), *PluginFileName());
}

FString FRustLoader::PluginFileName()
{
	return FString::Printf(TEXT("%s.%s"), TEXT("unreal_rust_loader"), *PlatformExtensionName());
}

FString FRustLoader::PluginPdbPath()
{
	return FPaths::Combine(*PluginFolderPath(), TEXT("rustplugin.pdb"));
}

FRustLoader::FRustLoader()
{
}

bool FRustLoader::IsRustOutOfDate() const
{
	return IsOutOfDateFunction() > 0;
}

void FRustLoader::LoadRust()
{
	if (IsLoaded())
	{
		// RustModule.Plugin.EditorTick(DeltaTime);
		TRACE_CPUPROFILER_EVENT_SCOPE(Hotreload);
		Types.Reset();
		if (TryLoadFunction(&Rust))
		{
			UE_LOG(LogTemp, Warning, TEXT("Hotreload"));
		}
		RegisterTypes();
	}
}

bool FRustLoader::SetupLoader()
{
	// Loading ddls is a bit tricky, see https://fasterthanli.me/articles/so-you-want-to-live-reload-rust
	// The gist is we can't easily hot reload a dll if the dll uses the thread local storage (TLS).
	// The TLS will prevent the dll from being unloaded even when we call `dlclose`. And `dlopen` will return
	// the pointer to the previously loaded dll.
	// Essentially this means hot reloading will do nothing as we can't unload the currently loaded dll.
	// The workaround for this is give each dll a unique name. Here we append the unix timestamp at
	// the end of the file. That way we can force `dlopen` to load the dll.
	// Please note this is a hack, and this _should_ leak and increase the memory every time you hot reload.
	// This behavior is the same on Linux, Windows and most likely all the other platforms.

	FString Path = PluginPath();
	//FString PdbPath = PluginPdbPath();
	FString LocalTargetDllPath = FPaths::Combine(*PluginFolderPath(), PluginFileName());
	if (this->IsLoaded())
	{
		FPlatformProcess::FreeDllHandle(this->Handle);
		this->Handle = nullptr;
	}

	void* LocalHandle = FPlatformProcess::GetDllHandle(*LocalTargetDllPath);

	if (LocalHandle == nullptr)
	{
		UE_LOG(LogTemp, Warning, TEXT("Dll open failed"));
		return false;
	}

	this->Handle = LocalHandle;

	this->Bindings = reinterpret_cast<EntryUnrealBindingsFn>(FPlatformProcess::GetDllExport(LocalHandle, TEXT("register_unreal_bindings\0")));
	this->TryLoadFunction = reinterpret_cast<TryLoadFn>(FPlatformProcess::GetDllExport(LocalHandle, TEXT("try_load\0")));
	this->IsOutOfDateFunction = reinterpret_cast<IsOutOfDateFn>(FPlatformProcess::GetDllExport(LocalHandle, TEXT("is_out_of_date\0")));
	ensure(this->Bindings);
	ensure(this->TryLoadFunction);

	this->TargetPath = LocalTargetDllPath;
	NeedsInit = true;
	CallEntryPoints();
	LoadRust();
	return true;
}

void FRustPluginModule::Exit()
{
	if (GEditor)
	{
		GEditor->RequestEndPlayMap();
	}
}

bool FRustLoader::IsLoaded()
{
	return Handle != nullptr;
}

void FRustLoader::CallEntryPoints()
{
	if (!IsLoaded())
		return;

	// Pass unreal function pointers to Rust, and also retrieve function pointers from Rust so we can call into Rust
	auto UnrealBindings = CreateBindings();
	if (Bindings(UnrealBindings))
	{
		//Rust.initialize_modules();
	}
	else
	{
		// TODO: We had a panic when calling the entry point. We need to handle that better, otherwise unreal will segfault because the rust bindings are nullptrs
	}
}

UE_DISABLE_OPTIMIZATION

void FRustLoader::RegisterTypes()
{
	auto& Module = GetRustModule();
	FArchive ArDummy;

	TUniquePtr<FReload> Reload(new FReload(EActiveReloadType::Reinstancing, TEXT(""), *GLog));
	TSet<UClass*> OldClasses;
	for (auto& It : Types)
	{
		UClass* OldClass = FindObject<UClass>(Module.GetPackage(), *It.Key);
		if (OldClass != nullptr)
		{
			// TODO: We might need to make the name unique. If we fail to delete the replace class later then this will
			// crash here
			FString OldClassName = FString::Printf(TEXT("%s_REPLACED_%i"), *It.Key, UniqueClassId);
			OldClass->Rename(*OldClassName, nullptr, REN_DontCreateRedirectors);
			OldClass->ClassFlags |= CLASS_NewerVersionExists;
			OldClasses.Add(OldClass);
			UniqueClassId += 1;
		}

		UClass* NewClass = NewObject<UClass>(
			Module.GetPackage(),
			UClass::StaticClass(),
			FName(*It.Key),
			RF_Public | RF_Standalone | RF_MarkAsRootSet
		);

		UClass* SuperClass = UDataAsset::StaticClass();
		NewClass->PropertyLink = SuperClass->PropertyLink;
		NewClass->SetSuperStruct(SuperClass);
		NewClass->ClassConstructor = SuperClass->ClassConstructor;
		NewClass->CppClassStaticFunctions = SuperClass->CppClassStaticFunctions;

		for (auto& PropertyDefinition : It.Value.PropertyDefinitions)
		{
			const FName PropertyName(*PropertyDefinition.Name);
			FProperty* Property = CreatePropertyFromRustType(
				NewClass,
				PropertyName,
				RF_Public,
				PropertyDefinition.Type.Get());

			if (Property == nullptr)
			{
				continue;
			}

			Property->SetPropertyFlags(ResolveSpecifiers(PropertyDefinition.Specifiers));
			for (auto& Meta : PropertyDefinition.Metadata)
			{
				Property->SetMetaData(*Meta.Key, *Meta.Value);
			}

			// HACK: We can't set the offset for properties directly for some reason. Instead we trick unreal
			// by setting the property sizes and then link. Stolen from the angelscript bindings
			// Later we set the correct property sizes again
			NewClass->AddCppProperty(Property);
			NewClass->SetPropertiesSize(PropertyDefinition.Offset);
			Property->Link(ArDummy);
			check(Property->GetOffset_ForUFunction() == PropertyDefinition.Offset);
		}

		NewClass->SetPropertiesSize(It.Value.Size);
		NewClass->StaticLink(false);
		NotifyRegistrationEvent(TEXT("/Script/Rust"), *NewClass->GetName(), ENotifyRegistrationType::NRT_Class,
		                        ENotifyRegistrationPhase::NRP_Finished, nullptr, false, NewClass);

		if (OldClass != nullptr && NewClass != nullptr)
		{
			Reload->NotifyChange(NewClass, OldClass);
		}
	}

	Reload->Reinstance();

	// // Remove old classes
	// for (auto Old : OldClasses)
	// {
	// 	Old->RemoveFromRoot();
	// 	Old->ClearFlags(RF_Standalone | RF_Public);
	// }
	//
	// // force the GC to run and remove the fully
	// CollectGarbage(GARBAGE_COLLECTION_KEEPFLAGS, true);
}

UE_ENABLE_OPTIMIZATION

void FRustPluginModule::StartupModule()
{
	// TODO: Don't run the module if we just want to generate the api. We should allow this and properly handle loading of this module if we don't have
	// a valid rust dll yet.
	FString RunCommand;
	if (FParse::Value(FCommandLine::Get(), TEXT("run="), RunCommand) && RunCommand == TEXT("GenerateApi"))
	{
		return;
	}
	FRustPluginStyle::Initialize();
	FRustPluginStyle::ReloadTextures();

	FRustPluginCommands::Register();

	RustPackage = NewObject<UPackage>(nullptr, FName(TEXT("/Script/Rust")),
	                                  RF_Public | RF_Standalone | RF_MarkAsRootSet);
	RustPackage->SetPackageFlags(PKG_CompiledIn);

	//PluginCommands = MakeShareable(new FUICommandList);

	//PluginCommands->MapAction(
	//    FRustPluginCommands::Get().OpenPluginWindow,
	//    FExecuteAction::CreateRaw(this, &FRustPluginModule::PluginButtonClicked),
	//    FCanExecuteAction());

	//UToolMenus::RegisterStartupCallback(FSimpleMulticastDelegate::FDelegate::CreateRaw(this, &FRustPluginModule::RegisterMenus));

	//FGlobalTabmanager::Get()->RegisterNomadTabSpawner(RustPluginTabName, FOnSpawnTab::CreateRaw(this, &FRustPluginModule::OnSpawnPluginTab)).SetDisplayName(LOCTEXT("FRustPluginTabTitle", "RustPlugin")).SetMenuType(ETabSpawnerMenuType::Hidden);

	//IDirectoryWatcher* watcher = FModuleManager::LoadModuleChecked<FDirectoryWatcherModule>(
	//		TEXT("DirectoryWatcher")).
	//	Get();
	//watcher->RegisterDirectoryChangedCallback_Handle(
	//	*Plugin.PluginFolderPath(),
	//	IDirectoryWatcher::FDirectoryChanged::CreateRaw(this, &FRustPluginModule::OnProjectDirectoryChanged),
	//	WatcherHandle, IDirectoryWatcher::WatchOptions::IgnoreChangesInSubtree);
	Plugin.SetupLoader();

	//TSharedPtr<FUuidGraphPanelPinFactory> UuidFactory = MakeShareable(new FUuidGraphPanelPinFactory());
	//FEdGraphUtilities::RegisterVisualPinFactory(UuidFactory);

	// Register detail customizations
	{
		auto& PropertyModule = FModuleManager::LoadModuleChecked<FPropertyEditorModule>("PropertyEditor");

		//PropertyModule.RegisterCustomClassLayout(
		//	"EntityComponent",
		//	FOnGetDetailCustomizationInstance::CreateStatic(&FRustDetailCustomization::MakeInstance)
		//);

		//PropertyModule.RegisterCustomPropertyTypeLayout(
		//	FRustEvent::StaticStruct()->GetFName(),
		//	FOnGetPropertyTypeCustomizationInstance::CreateStatic(&FRustAnimNotifyDetailCustomization::MakeInstance)
		//);

		//PropertyModule.RegisterCustomClassLayout(
		//	"AnimNotify_RustEvent",
		//	FOnGetDetailCustomizationInstance::CreateStatic(&FRustAnimNotifyDetailCustomization::MakeInstance)
		//);

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
			Plugin.SetupLoader();
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

UWorld* URustEditorSubsystem::GetWorld() const
{
	return GEditor->GetEditorWorldContext().World();
}

void URustEditorSubsystem::Tick(float DeltaTime)
{
	auto& RustModule = GetRustModule();
	if (RustModule.Plugin.IsLoaded() && RustModule.Plugin.IsRustOutOfDate())
	{
		RustModule.Plugin.LoadRust();
	}
}

void URustEditorSubsystem::Initialize(FSubsystemCollectionBase& Collection)
{
	Super::Initialize(Collection);
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
			[
				SNew(STextBlock)
				.Text(WidgetText)
			]
		];
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
