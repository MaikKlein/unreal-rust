// Fill out your copyright notice in the Description page of Project Settings.


#include "RustProperty.h"

#include "ContentBrowserModule.h"
#include "RustPlugin.h"
#include "RustUtils.h"
#include "DetailWidgetRow.h"
#include "IContentBrowserSingleton.h"
#include "PropertyCustomizationHelpers.h"
#include "EditorWidgets/Public/SAssetDropTarget.h"
#include "GameFramework/GameModeBase.h"
#include "Widgets/Input/SVectorInputBox.h"

#define LOCTEXT_NAMESPACE "RustProperty"

void FDynamicRustComponent::Initialize(TSharedPtr<IPropertyHandle> Handle, FGuid InitGuid)
{
	TSharedPtr<IPropertyHandle> NameProperty = Handle->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Name));
	TSharedPtr<IPropertyHandle> RustPropertyMap = Handle->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Fields));

	auto Fns = &GetModule().Plugin.Rust.reflection_fns;
	//Guid = InitGuid;
	Uuid Id = ToUuid(InitGuid);

	const char* TypeNamePtr = nullptr;
	uintptr_t TypeNameLen = 0;
	Fns->get_type_name(Id, &TypeNamePtr, &TypeNameLen);
	FString Name = FString(TypeNameLen, UTF8_TO_TCHAR(TypeNamePtr));
	UE_LOG(LogTemp, Warning, TEXT("Name %s"), *Name);
	NameProperty->SetValue(Name);

	uint32_t NumberOfFields = 0;
	Fns->number_of_fields(Id, &NumberOfFields);

	for (uint32_t Idx = 0; Idx < NumberOfFields; ++Idx)
	{
		const char* NamePtr = nullptr;
		uintptr_t Len = 0;
		Fns->get_field_name(Id, Idx, &NamePtr, &Len);
		FString FieldName = FString(Len, UTF8_TO_TCHAR(NamePtr));

		ReflectionType Type;
		Fns->get_field_type(Id, Idx, &Type);


		uint32 FieldIndex = 0;
		RustPropertyMap->GetNumChildren(FieldIndex);
		RustPropertyMap->AsMap()->AddItem();

		auto RustPropertyEntry = RustPropertyMap->GetChildHandle(FieldIndex);
		RustPropertyEntry->GetKeyHandle()->SetValue(FieldName);
		auto HandleTag = RustPropertyEntry->GetChildHandle(GET_MEMBER_NAME_CHECKED(FRustProperty, Tag));
		// How does that not work? Falling back to an int32 instead
		// HandleTag->SetValue(TEnumAsByte<ERustPropertyTag>(ERustPropertyTag::Float));
		if (Type == ReflectionType::Float)
		{
			HandleTag->SetValue(ERustPropertyTag::Float);
			auto HandleFloat = RustPropertyEntry->GetChildHandle(GET_MEMBER_NAME_CHECKED(FRustProperty, Float));
		}
		if (Type == ReflectionType::Vector3)
		{
			HandleTag->SetValue(ERustPropertyTag::Vector);
		}
		if (Type == ReflectionType::Bool)
		{
			HandleTag->SetValue(ERustPropertyTag::Bool);
		}
		if (Type == ReflectionType::Quaternion)
		{
			HandleTag->SetValue(ERustPropertyTag::Quat);
		}
		if (Type == ReflectionType::UClass)
		{
			HandleTag->SetValue(ERustPropertyTag::Class);
		}
		if (Type == ReflectionType::USound)
		{
			HandleTag->SetValue(ERustPropertyTag::Sound);
		}
	}
}

void FDynamicRustComponent::Render(TSharedRef<IPropertyHandle> MapHandle, IDetailCategoryBuilder& DetailBuilder,
                                    const TSharedRef<class IPropertyUtilities> Utilities,
                                    FOnComponentRemoved OnComponentRemoved)
{
	uint32 NumberOfComponents;
	MapHandle->GetNumChildren(NumberOfComponents);
	for (uint32 ComponentIdx = 0; ComponentIdx < NumberOfComponents; ++ComponentIdx)
	{
		auto ComponentEntry = MapHandle->GetChildHandle(ComponentIdx);
		TSharedPtr<IPropertyHandle> NameProperty = ComponentEntry->GetChildHandle(
			GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Name));

		if (!NameProperty.IsValid())
		{
			continue;
		}

		FString Name;
		NameProperty->GetValue(Name);

		auto RowText = LOCTEXT("RustCategory", "Components");
		DetailBuilder.AddCustomRow(RowText).WholeRowContent()[
			SNew(SBorder)
				.BorderBackgroundColor(FSlateColor(FColor(0.0, 0.0, 0.0, 1.0)))
				.Padding(0)
			[
				SNew(SHorizontalBox) +
				SHorizontalBox::Slot()[
					SNew(STextBlock).Text(FText::FromString(Name))
				]
				+ SHorizontalBox::Slot()[
					SNew(SBox)
				.HAlign(HAlign_Center)
				.VAlign(VAlign_Center)
				.WidthOverride(22)
				.HeightOverride(22)[
						SNew(SButton)
					.ButtonStyle(FAppStyle::Get(), "SimpleButton")
					.OnClicked(OnComponentRemoved)
					.ContentPadding(0)
						//.IsFocusable(InArgs._IsFocusable)
						[
							SNew(SImage)
						.Image(FEditorStyle::GetBrush("Icons.Delete"))
						.ColorAndOpacity(FSlateColor::UseForeground())
						]
					]
				]
			]
		];

		TSharedPtr<IPropertyHandle> FieldsProperty = ComponentEntry->GetChildHandle(
			GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Fields));

		uint32 NumberOfFields = 0;
		FieldsProperty->GetNumChildren(NumberOfFields);

		for (uint32 FieldIdx = 0; FieldIdx < NumberOfFields; ++FieldIdx)
		{
			auto RustPropertyEntry = FieldsProperty->GetChildHandle(FieldIdx);
			TSharedPtr<IPropertyHandle> TagProperty = RustPropertyEntry->GetChildHandle(
				GET_MEMBER_NAME_CHECKED(FRustProperty, Tag));

			int32 Tag = 0;
			TagProperty->GetValue(Tag);

			TSharedPtr<IPropertyHandle> FieldNameProperty = RustPropertyEntry->GetKeyHandle();

			FString FieldPropertyName;
			FieldNameProperty->GetValue(FieldPropertyName);

			auto& W = DetailBuilder.AddCustomRow(RowText).NameContent()
			[
				FieldNameProperty->CreatePropertyNameWidget(FText::FromString(FieldPropertyName))
				//SNew(STextBlock).Text(FText::FromString(FieldPropertyName))
			];
			if (Tag == ERustPropertyTag::Float)
			{
				auto FloatProperty = RustPropertyEntry->GetChildHandle(
					GET_MEMBER_NAME_CHECKED(FRustProperty, Float));
				W.ValueContent()[
					FloatProperty->CreatePropertyValueWidget()
				];
			}
			if (Tag == ERustPropertyTag::Vector)
			{
				auto VectorProperty = RustPropertyEntry->GetChildHandle(
					GET_MEMBER_NAME_CHECKED(FRustProperty, Vector));
				UE_LOG(LogTemp, Warning, TEXT("Vector %i"), VectorProperty.IsValid());
				auto SetVector = [=](FVector V)
				{
					const FScopedTransaction Transaction(LOCTEXT("Vector", "Store"));
					VectorProperty->SetValue(V);
				};
				auto SetVectorCommitted = [=](FVector V, ETextCommit::Type Ty)
				{
					const FScopedTransaction Transaction(LOCTEXT("Vector", "Store"));
					VectorProperty->SetValue(V);
				};
				auto GetValue = [=]() -> TOptional<FVector>
				{
					FVector V;
					VectorProperty->GetValue(V);
					return TOptional(V);
				};
				auto OnXChanged = [=](double Val)
				{
					FVector V;
					VectorProperty->GetValue(V);
					V.X = Val;
					VectorProperty->SetValue(V);
				};
				auto OnYChanged = [=](double Val)
				{
					FVector V;
					VectorProperty->GetValue(V);
					V.Y = Val;
					VectorProperty->SetValue(V);
				};
				auto OnZChanged = [=](double Val)
				{
					FVector V;
					VectorProperty->GetValue(V);
					V.Z = Val;
					VectorProperty->SetValue(V);
				};
				W.ValueContent()
				 .MinDesiredWidth(125.0f * 3.0f)
				 .MaxDesiredWidth(125.0f * 3.0f)
				 .VAlign(VAlign_Center)
				[
					SNew(SNumericVectorInputBox<FVector::FReal>)
				.AllowSpin(true)
				.Vector_Lambda(GetValue)
				.OnVectorChanged_Lambda(SetVector)
				.OnXChanged_Lambda(OnXChanged)
				.OnYChanged_Lambda(OnYChanged)
				.OnZChanged_Lambda(OnZChanged)
			.OnVectorCommitted_Lambda(SetVectorCommitted)
			.bColorAxisLabels(true)
				];
			}
			if (Tag == ERustPropertyTag::Bool)
			{
				auto BoolProperty = RustPropertyEntry->GetChildHandle(
					GET_MEMBER_NAME_CHECKED(FRustProperty, Bool));
				W.ValueContent()[
					BoolProperty->CreatePropertyValueWidget()
				];
			}
			if (Tag == ERustPropertyTag::Quat)
			{
				auto BoolProperty = RustPropertyEntry->GetChildHandle(
					GET_MEMBER_NAME_CHECKED(FRustProperty, Rotation));
				W.ValueContent()[
					BoolProperty->CreatePropertyValueWidget()
				];
			}
			if (Tag == ERustPropertyTag::Class)
			{
				auto BoolProperty = RustPropertyEntry->GetChildHandle(
					GET_MEMBER_NAME_CHECKED(FRustProperty, Class));
				W.ValueContent()[
					BoolProperty->CreatePropertyValueWidget()
				];
			}
			if (Tag == ERustPropertyTag::Sound)
			{
				auto Property = RustPropertyEntry->GetChildHandle(
					GET_MEMBER_NAME_CHECKED(FRustProperty, Sound));
				W.ValueContent()[
					Property->CreatePropertyValueWidget()
				];
			}
		}
	}
}

//void UDynamicRustComponent::Initialize(FGuid InitGuid, UObject* Owner)
//{
//	auto Fns = &GetModule().Plugin.Rust.reflection_fns;
//	Guid = InitGuid;
//
//	Uuid Id = ToUuid(Guid);
//
//	const char* TypeNamePtr = nullptr;
//	uintptr_t TypeNameLen = 0;
//	Fns->get_type_name(Id, &TypeNamePtr, &TypeNameLen);
//	Name = FString(TypeNameLen, UTF8_TO_TCHAR(TypeNamePtr));
//
//	uint32_t NumberOfFields = 0;
//	Fns->number_of_fields(Id, &NumberOfFields);
//
//	for (uint32_t Idx = 0; Idx < NumberOfFields; ++Idx)
//	{
//		const char* NamePtr = nullptr;
//		uintptr_t Len = 0;
//		Fns->get_field_name(Id, Idx, &NamePtr, &Len);
//		FString FieldName = FString(Len, UTF8_TO_TCHAR(NamePtr));
//
//		ReflectionType Type;
//		Fns->get_field_type(Id, Idx, &Type);
//
//
//		if (Type == ReflectionType::Vector3)
//		{
//			Fields.Add(FieldName, NewObject<URustPropertyVector>(Owner));
//		}
//
//		if (Type == ReflectionType::Bool)
//		{
//			Fields.Add(FieldName, NewObject<URustPropertyBool>(Owner));
//		}
//		if (Type == ReflectionType::Float)
//		{
//			Fields.Add(FieldName, NewObject<URustPropertyFloat>(Owner));
//		}
//		if (Type == ReflectionType::Quaternion)
//		{
//			Fields.Add(FieldName, NewObject<URustPropertyQuaternion>(Owner));
//		}
//		if (Type == ReflectionType::UClass)
//		{
//			auto Prop = NewObject<URustPropertyUClass>(Owner);
//			Fields.Add(FieldName, Prop);
//		}
//	}
//}

//void UDynamicRustComponent::Render(TSharedRef<IPropertyHandle> MapHandle, IDetailCategoryBuilder& DetailBuilder,
//                                   FOnComponentRemoved OnComponentRemoved)
//{
//	auto RowText = LOCTEXT("RustCategory", "Components");
//	DetailBuilder.AddCustomRow(RowText).WholeRowContent()[
//		SNew(SBorder)
//				.BorderBackgroundColor(FSlateColor(FColor(0.0, 0.0, 0.0, 1.0)))
//				.Padding(0)
//		[
//			SNew(SHorizontalBox) +
//			SHorizontalBox::Slot()[
//				SNew(STextBlock).Text(FText::FromString(Name))
//			]
//			+ SHorizontalBox::Slot()[
//				SNew(SBox)
//				.HAlign(HAlign_Center)
//				.VAlign(VAlign_Center)
//				.WidthOverride(22)
//				.HeightOverride(22)[
//					SNew(SButton)
//					.ButtonStyle(FAppStyle::Get(), "SimpleButton")
//					.OnClicked(OnComponentRemoved)
//					.ContentPadding(0)
//					//.IsFocusable(InArgs._IsFocusable)
//					[
//						SNew(SImage)
//						.Image(FEditorStyle::GetBrush("Icons.Delete"))
//						.ColorAndOpacity(FSlateColor::UseForeground())
//					]
//				]
//			]
//		]
//	];
//
//	//uint32 NumChildren = 0;
//	//MapHandle->GetNumChildren(NumChildren);
//
//	//for (uint32 Idx = 0; Idx < NumChildren; ++Idx)
//	//{
//	//	auto ChildProp = MapHandle->GetChildHandle(Idx);
//	//	auto FieldProp = ChildProp->GetChildHandle(GET_MEMBER_NAME_CHECKED(FDynamicRustComponent2, Fields));
//	//	uint32 NumFieldChildren = 0;
//	//	FieldProp->GetNumChildren(NumFieldChildren);
//
//	//	UE_LOG(LogTemp, Warning, TEXT("Children %i"), NumFieldChildren);
//	//	for (uint32 Idx2 = 0; Idx2 < NumFieldChildren; ++Idx2)
//	//	{
//	//		auto RustProp = FieldProp->GetChildHandle(Idx2);
//
//	//			TArray<void*> RawData;
//	//			RustProp->AccessRawData(RawData);
//	//			FRustProperty2* Val = static_cast<FRustProperty2*>(RawData[0]);
//	//		auto Foo = RustProp->GetChildHandle(GET_MEMBER_NAME_CHECKED(FRustProperty2, Foo));
//	//		auto Bar = RustProp->GetChildHandle(GET_MEMBER_NAME_CHECKED(FRustProperty2, Bar));
//
//	//		UE_LOG(LogTemp, Warning, TEXT("FooBar %i %i"), Foo.IsValid(), Bar.IsValid());
//	//		UE_LOG(LogTemp, Warning, TEXT("Foo Bar %i %i"), Val->Foo, Val->Bar);
//	//	}
//	//}
//
//	// TODO: This is how the property system should work. But for some reason we can't access `URustDynamicComponent` as a property
//	//       We do get back the correct value from `GetValue`, but the returned property has no fields.
//	//       Leaving the code snippet here for future use
//
//	//uint32 NumChildren = 0;
//	//MapHandle->GetNumChildren(NumChildren);
//
//	//for (uint32 Idx = 0; Idx < NumChildren; ++Idx)
//	//{
//	//	auto ChildProp = MapHandle->GetChildHandle(Idx);
//	//	auto FieldProp = ChildProp->GetChildHandle(GET_MEMBER_NAME_CHECKED(UDynamicRustComponent, Fields));
//
//	//	UObject* Val;
//	//	ChildProp->GetValue(Val);
//	//	if(Cast<UDynamicRustComponent>(Val) != nullptr)
//	//	{
//	//		UE_LOG(LogTemp, Warning, TEXT("works"));
//	//	}
//	//	//auto KeyProp = ChildProp->GetKeyHandle();
//
//	//	//FString GuidKey;
//	//	//KeyProp->GetValue(GuidKey);
//
//
//	//	uint32 NumPropChildren = 0;
//	//	ChildProp->GetNumChildren(NumPropChildren);
//	//	UE_LOG(LogTemp, Warning, TEXT("%s %i %i"), *ChildProp->GetPropertyClass()->GetName(), NumPropChildren, FieldProp.IsValid());
//
//	//	//auto Data = ChildProp->GetChildHandle(0);
//	//	DetailBuilder.AddCustomRow(RowText)
//	//	.ValueContent()[
//	//		ChildProp->CreatePropertyValueWidget()
//	//	];
//	//}
//
//	for (auto& Elem : Fields)
//	{
//		auto BoolProp = Cast<URustPropertyBool>(Elem.Value);
//		auto VectorProp = Cast<URustPropertyVector>(Elem.Value);
//		auto QuatProp = Cast<URustPropertyQuaternion>(Elem.Value);
//		auto FloatProp = Cast<URustPropertyFloat>(Elem.Value);
//		auto UClassProp = Cast<URustPropertyUClass>(Elem.Value);
//
//		TSharedRef<SWidget> Widget = SNew(SCheckBox);
//
//		auto& W = DetailBuilder.AddCustomRow(RowText).NameContent()
//		[
//			SNew(STextBlock).Text(FText::FromString(Elem.Key))
//		];
//		if (BoolProp != nullptr)
//		{
//			auto Value = [=]() -> ECheckBoxState
//			{
//				if (BoolProp->Data)
//				{
//					return ECheckBoxState::Checked;
//				}
//				else
//				{
//					return ECheckBoxState::Unchecked;
//				}
//			};
//			auto OnChanged = [=](ECheckBoxState Changed)
//			{
//				if (Changed == ECheckBoxState::Checked)
//				{
//					BoolProp->Data = true;
//					return;
//				}
//				BoolProp->Data = false;
//			};
//			W.ValueContent()[
//				SNew(SCheckBox).IsChecked_Lambda(Value).OnCheckStateChanged_Lambda(OnChanged)
//			];
//		}
//		if (FloatProp != nullptr)
//		{
//			auto OnChanged = [=](float Val)
//			{
//				FloatProp->Data = Val;
//			};
//			auto Value = [=]() -> float
//			{
//				return FloatProp->Data;
//			};
//			W.ValueContent()[
//				SNew(SNumericEntryBox<float>).OnValueChanged_Lambda(OnChanged).Value_Lambda(Value)
//			];
//		}
//		if (QuatProp != nullptr)
//		{
//			auto Roll = [=]() -> TOptional<FRotator::FReal>
//			{
//				return QuatProp->Data.Rotator().Roll;
//			};
//			auto Yaw = [=]() -> TOptional<FRotator::FReal>
//			{
//				return QuatProp->Data.Rotator().Yaw;
//			};
//			auto Pitch = [=]() -> TOptional<FRotator::FReal>
//			{
//				return QuatProp->Data.Rotator().Pitch;
//			};
//			auto OnRollChanged = [=](FRotator::FReal Val)
//			{
//				FRotator R = QuatProp->Data.Rotator();
//				R.Roll = Val;
//				QuatProp->Data = R.Quaternion();
//			};
//			auto OnYawChanged = [=](FRotator::FReal Val)
//			{
//				FRotator R = QuatProp->Data.Rotator();
//				R.Yaw = Val;
//				QuatProp->Data = R.Quaternion();
//			};
//			auto OnPitchChanged = [=](FRotator::FReal Val)
//			{
//				FRotator R = QuatProp->Data.Rotator();
//				R.Pitch = Val;
//				QuatProp->Data = R.Quaternion();
//			};
//			TSharedPtr<TNumericUnitTypeInterface<FRotator::FReal>> DegreesTypeInterface =
//				MakeShareable(new TNumericUnitTypeInterface<FRotator::FReal>(EUnit::Degrees));
//			W.ValueContent()
//			 .MinDesiredWidth(125.0f * 3.0f)
//			 .MaxDesiredWidth(125.0f * 3.0f)
//			 .VAlign(VAlign_Center)
//			[
//				SNew(SNumericRotatorInputBox<FRotator::FReal>)
//				.TypeInterface(DegreesTypeInterface)
//				.AllowSpin(true).bColorAxisLabels(true)
//				.Pitch_Lambda(Pitch)
//				.Yaw_Lambda(Yaw)
//				.Roll_Lambda(Roll)
//				.OnRollChanged_Lambda(OnRollChanged)
//				.OnPitchChanged_Lambda(OnPitchChanged)
//				.OnYawChanged_Lambda(OnYawChanged)
//			];
//		}
//		if (VectorProp != nullptr)
//		{
//			auto SetVector = [=](FVector V)
//			{
//				VectorProp->Data = V;
//				UE_LOG(LogTemp, Warning, TEXT("Changed %s"), *VectorProp->Data.ToString());
//			};
//			auto SetVectorCommitted = [=](FVector V, ETextCommit::Type Ty)
//			{
//				VectorProp->Data = V;
//				UE_LOG(LogTemp, Warning, TEXT("Commit %s"), *VectorProp->Data.ToString());
//			};
//			auto GetValue = [=]() -> TOptional<FVector>
//			{
//				return TOptional(VectorProp->Data);
//			};
//			auto OnXChanged = [=](double Val)
//			{
//				VectorProp->Data.X = Val;
//			};
//			auto OnYChanged = [=](double Val)
//			{
//				VectorProp->Data.Y = Val;
//			};
//			auto OnZChanged = [=](double Val)
//			{
//				VectorProp->Data.Z = Val;
//			};
//			W.ValueContent()
//			 .MinDesiredWidth(125.0f * 3.0f)
//			 .MaxDesiredWidth(125.0f * 3.0f)
//			 .VAlign(VAlign_Center)
//			[
//				SNew(SNumericVectorInputBox<FVector::FReal>)
//				.AllowSpin(true)
//				.Vector_Lambda(GetValue)
//				.OnVectorChanged_Lambda(SetVector)
//				.OnXChanged_Lambda(OnXChanged)
//				.OnYChanged_Lambda(OnYChanged)
//				.OnZChanged_Lambda(OnZChanged)
//			.OnVectorCommitted_Lambda(SetVectorCommitted)
//			.bColorAxisLabels(true)
//			];
//		}
//		if (UClassProp != nullptr)
//		{
//			FContentBrowserModule& ContentBrowserModule = FModuleManager::Get().LoadModuleChecked<
//				FContentBrowserModule>(TEXT("ContentBrowser"));
//
//			FAssetPickerConfig AssetPickerConfig;
//			{
//				//AssetPickerConfig.OnAssetSelected = FOnAssetSelected::CreateRaw(this, &FMediaTrackEditor::AddNewSection, MediaTrack);
//				//AssetPickerConfig.OnAssetEnterPressed = FOnAssetEnterPressed::CreateRaw(this, &FMediaTrackEditor::AddNewSectionEnterPressed, MediaTrack);
//				AssetPickerConfig.bAllowNullSelection = false;
//				AssetPickerConfig.InitialAssetViewType = EAssetViewType::List;
//				AssetPickerConfig.Filter.bRecursiveClasses = true;
//				//AssetPickerConfig.Filter.ClassNames.Add(URustProperty::StaticClass()->GetFName());
//				//AssetPickerConfig.Filter.ClassNames.Add(AActor::StaticClass()->GetFName());
//				//AssetPickerConfig.Filter.ClassNames.Add(ARustActor::StaticClass()->GetFName());
//				//AssetPickerConfig.Filter.ClassNames.Add(USoundCue::StaticClass()->GetFName());
//				//AssetPickerConfig.Filter.ClassNames.Add(URustProperty::StaticClass()->GetFName());
//				AssetPickerConfig.bForceShowPluginContent = true;
//				//AssetPickerConfig.bForceShowEngineContent = true;
//				AssetPickerConfig.bCanShowReadOnlyFolders = true;
//				AssetPickerConfig.bCanShowClasses = true;
//				AssetPickerConfig.OnShouldFilterAsset = FOnShouldFilterAsset::CreateLambda(
//					[](const FAssetData& Data)
//					{
//						return !Data.GetClass()->IsChildOf(UAnimationAsset::StaticClass());
//					});
//			}
//			FClassViewerInitializationOptions InitOptions;
//			InitOptions.bIsActorsOnly = true;
//			InitOptions.InitiallySelectedClass = UClassProp->Data;
//
//			auto ClassViewer = StaticCastSharedRef<SClassViewer>(
//				FModuleManager::LoadModuleChecked<FClassViewerModule>("ClassViewer").CreateClassViewer(
//					InitOptions, FOnClassPicked::CreateLambda([UClassProp](UClass* PickedClass)
//					{
//						UClassProp->Data = PickedClass;
//						FSlateApplication::Get().DismissAllMenus();
//					})));
//			W.ValueContent()[
//				SNew(SComboButton).MenuContent()[
//					//ContentBrowserModule.Get().CreateAssetPicker(AssetPickerConfig)
//					ClassViewer
//				]
//				.ButtonContent()
//				[
//					SNew(STextBlock)
//					.Text_Lambda([UClassProp]()
//					{
//						FText ClassName;
//						if (UClassProp->Data != nullptr)
//						{
//							ClassName = FText::FromString(UClassProp->Data->GetName());
//						}
//						return ClassName;
//					})
//				]
//			];
//		}
//	}
//}

#undef LOCTEXT_NAMESPACE
