// Fill out your copyright notice in the Description page of Project Settings.

#include "RustProperty.h"

#include "DetailLayoutBuilder.h"
#include "RustPlugin.h"
#include "RustUtils.h"
#include "DetailWidgetRow.h"
#include "IContentBrowserSingleton.h"
#include "IDetailGroup.h"
#include "JsonObjectConverter.h"
#include "Dom/JsonObject.h"
#include "EditorWidgets/Public/SAssetDropTarget.h"
#include "GameFramework/GameModeBase.h"
#include "Widgets/Input/SVectorInputBox.h"
#include "Widgets/Layout/SBox.h"
#include "Widgets/Input//SButton.h"

#define LOCTEXT_NAMESPACE "RustProperty"

void FRustProperty::Initialize(TSharedPtr<IPropertyHandle> Handle, ReflectionType Type)
{
	auto HandleTag = Handle->GetChildHandle(GET_MEMBER_NAME_CHECKED(FRustProperty, Tag));
	// How does that not work? Falling back to an int32 instead
	// HandleTag->SetValue(TEnumAsByte<ERustPropertyTag>(ERustPropertyTag::Float));
	if (Type == ReflectionType::Float)
	{
		HandleTag->SetValue(ERustPropertyTag::Float);
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

void FDynamicRustComponent::Reload(TSharedPtr<IPropertyHandle> Handle, FGuid Guid)
{
	TSharedPtr<IPropertyHandle> FieldsProperty = Handle->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Fields));

	Uuid Id = ToUuid(Guid);

	auto Reflection = GetRustModule().Plugin.ReflectionData.Types.Find(Guid);
	check(Reflection);
	for (auto& Elem : Reflection->FieldNameToType)
	{
		if (Fields.Find(Elem.Key) == nullptr)
		{
			uint32 IndexAdded = 0;
			FieldsProperty->GetNumChildren(IndexAdded);
			FieldsProperty->AsMap()->AddItem();

			auto RustPropertyEntry = FieldsProperty->GetChildHandle(IndexAdded);
			FRustProperty::Initialize(RustPropertyEntry, Elem.Value);
			auto KeyHandle = RustPropertyEntry->GetKeyHandle();
			KeyHandle->SetValue(Elem.Key);
		}
	}
}

void FDynamicRustComponent::Initialize(TSharedPtr<IPropertyHandle> Handle, FGuid InitGuid)
{
	TSharedPtr<IPropertyHandle> NameProperty = Handle->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Name));
	TSharedPtr<IPropertyHandle> RustPropertyMap = Handle->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Fields));

	auto Map = RustPropertyMap->AsMap();

	uint32 NumOfPreviousFields = 0;
	Map->GetNumElements(NumOfPreviousFields);
	for (uint32_t Idx = 0; Idx < NumOfPreviousFields; ++Idx)
	{
		Map->DeleteItem(Idx);
	}
	Map->Empty();
	Map->GetNumElements(NumOfPreviousFields);
	
	UE_LOG(LogTemp, Warning, TEXT("Fields after delete %i"), NumOfPreviousFields);
	if(NumOfPreviousFields != 0)
		return;
	

	Uuid Id = ToUuid(InitGuid);

	auto Reflection = GetRustModule().Plugin.ReflectionData.Types.Find(InitGuid);
	NameProperty->SetValue(Reflection->Name);

	uint32_t NumberOfFields = Reflection->IndexToFieldName.Num();

	for (uint32_t Idx = 0; Idx < NumberOfFields; ++Idx)
	{
		auto& FieldName = *Reflection->IndexToFieldName.Find(Idx);
		auto Type = *Reflection->FieldNameToType.Find(FieldName);

		uint32 FieldPropertyIndex = 0;
		Map->GetNumElements(FieldPropertyIndex);
		UE_LOG(LogTemp, Warning, TEXT("Field index %i"), FieldPropertyIndex);
		Map->AddItem();

		auto RustPropertyEntry = RustPropertyMap->GetChildHandle(FieldPropertyIndex);
		RustPropertyEntry->GetKeyHandle()->SetValue(FieldName);
		FRustProperty::Initialize(RustPropertyEntry, Type);
	}
}

void FDynamicRustComponent::RenderComponents(TSharedRef<IPropertyHandle> MapHandle,
                                             IDetailCategoryBuilder& DetailBuilder,
                                             IDetailLayoutBuilder& LayoutBuilder)
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

		TSharedPtr<IPropertyHandle> FieldsProperty = ComponentEntry->GetChildHandle(
			GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Fields));

		auto& ComponentGroup = DetailBuilder.AddGroup(FName("Component"), FText::FromString(Name), false, true);
		ComponentGroup.HeaderRow().NameContent()[
				FieldsProperty->CreatePropertyNameWidget(FText::FromString(Name))
			]
			.ExtensionContent()[
				SNew(SBox)
				.HAlign(HAlign_Center)
				.VAlign(VAlign_Center)
				.WidthOverride(22)
				.HeightOverride(22)[
					SNew(SButton)
					.ButtonStyle(FAppStyle::Get(), "SimpleButton")
					.OnClicked(FOnComponentRemoved::CreateLambda([MapHandle, ComponentIdx, &LayoutBuilder]()
					             {
						             MapHandle->AsMap()->DeleteItem(ComponentIdx);
						             LayoutBuilder.ForceRefreshDetails();
						             return FReply::Handled();
					             }))
					.ContentPadding(0)
					[
						SNew(SImage)
						.Image(FEditorStyle::GetBrush("Icons.Delete"))
						.ColorAndOpacity(FSlateColor::UseForeground())
					]
				]
			];
		FDynamicRustComponent::RenderFields(FieldsProperty, ComponentGroup);
	}
}

void FDynamicRustComponent::RenderFields(TSharedPtr<IPropertyHandle> FieldsProperty, IDetailGroup& ComponentGroup)

{
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
		if (Tag == ERustPropertyTag::Float)
		{
			auto FloatProperty = RustPropertyEntry->GetChildHandle(
				GET_MEMBER_NAME_CHECKED(FRustProperty, Float));
			ComponentGroup.AddPropertyRow(FloatProperty.ToSharedRef()).DisplayName(
				FText::FromString(FieldPropertyName));
		}
		if (Tag == ERustPropertyTag::Vector)
		{
			auto VectorProperty = RustPropertyEntry->GetChildHandle(
				GET_MEMBER_NAME_CHECKED(FRustProperty, Vector));
			ComponentGroup.AddPropertyRow(VectorProperty.ToSharedRef()).DisplayName(
				FText::FromString(FieldPropertyName));
		}
		if (Tag == ERustPropertyTag::Bool)
		{
			auto BoolProperty = RustPropertyEntry->GetChildHandle(
				GET_MEMBER_NAME_CHECKED(FRustProperty, Bool));
			ComponentGroup.AddPropertyRow(BoolProperty.ToSharedRef()).DisplayName(
				FText::FromString(FieldPropertyName));
		}
		if (Tag == ERustPropertyTag::Quat)
		{
			auto QuatProperty = RustPropertyEntry->GetChildHandle(
				GET_MEMBER_NAME_CHECKED(FRustProperty, Rotation));
			ComponentGroup.AddPropertyRow(QuatProperty.ToSharedRef()).DisplayName(
				FText::FromString(FieldPropertyName));
		}
		if (Tag == ERustPropertyTag::Class)
		{
			auto ClassProperty = RustPropertyEntry->GetChildHandle(
				GET_MEMBER_NAME_CHECKED(FRustProperty, Class));
			ComponentGroup.AddPropertyRow(ClassProperty.ToSharedRef()).DisplayName(
				FText::FromString(FieldPropertyName));
		}
		if (Tag == ERustPropertyTag::Sound)
		{
			auto SoundProperty = RustPropertyEntry->GetChildHandle(
				GET_MEMBER_NAME_CHECKED(FRustProperty, Sound));
			ComponentGroup.AddPropertyRow(SoundProperty.ToSharedRef()).DisplayName(
				FText::FromString(FieldPropertyName));
		}
	}
}

void FDynamicRustComponent::Render(TSharedRef<IPropertyHandle> Self, IDetailCategoryBuilder& DetailBuilder,
                                   IDetailLayoutBuilder& LayoutBuilder)
{
	TSharedPtr<IPropertyHandle> NameProperty = Self->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Name));

	if (!NameProperty.IsValid())
	{
		return;
	}

	FString Name;
	NameProperty->GetValue(Name);

	TSharedPtr<IPropertyHandle> FieldsProperty = Self->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FDynamicRustComponent, Fields));

	auto& ComponentGroup = DetailBuilder.AddGroup(FName("Component"), FText::FromString(Name), false, true);
	ComponentGroup.HeaderRow().NameContent()[
			FieldsProperty->CreatePropertyNameWidget(FText::FromString(Name))
		]
		.ExtensionContent()[
			SNew(SBox)
				.HAlign(HAlign_Center)
				.VAlign(VAlign_Center)
				.WidthOverride(22)
				.HeightOverride(22)[
				SNew(SButton)
					.ButtonStyle(FAppStyle::Get(), "SimpleButton")
					//.OnClicked(FOnComponentRemoved::CreateLambda([MapHandle, ComponentIdx, &LayoutBuilder]()
				    //         {
					//             MapHandle->AsMap()->DeleteItem(ComponentIdx);
					//             LayoutBuilder.ForceRefreshDetails();
					//             return FReply::Handled();
				    //         }))
					.ContentPadding(0)
				[
					SNew(SImage)
						.Image(FEditorStyle::GetBrush("Icons.Delete"))
						.ColorAndOpacity(FSlateColor::UseForeground())
				]
			]
		];
	FDynamicRustComponent::RenderFields(FieldsProperty, ComponentGroup);
}

FString FDynamicRustComponent::SerializeToJson()
{
	TSharedPtr<FJsonObject> Json = MakeShareable(new FJsonObject);
	for (auto Elem : this->Fields)
	{
		auto Tag = Elem.Value.Tag;

		if (Tag == ERustPropertyTag::Bool)
		{
			Json->SetBoolField(Elem.Key, Elem.Value.Bool);
		}
		if (Tag == ERustPropertyTag::Float)
		{
			Json->SetNumberField(Elem.Key, Elem.Value.Float);
		}
		if (Tag == ERustPropertyTag::Quat)
		{
			auto Quat = Elem.Value.Rotation.Quaternion();

			TArray<TSharedPtr<FJsonValue>> Array;
			Array.Push(MakeShared<FJsonValueNumber>(Quat.X));
			Array.Push(MakeShared<FJsonValueNumber>(Quat.Y));
			Array.Push(MakeShared<FJsonValueNumber>(Quat.Z));
			Array.Push(MakeShared<FJsonValueNumber>(Quat.W));
			Json->SetArrayField(Elem.Key, Array);
		}
		if (Tag == ERustPropertyTag::Vector)
		{
			auto Vector = Elem.Value.Vector;

			TArray<TSharedPtr<FJsonValue>> Array;
			Array.Push(MakeShared<FJsonValueNumber>(Vector.X));
			Array.Push(MakeShared<FJsonValueNumber>(Vector.Y));
			Array.Push(MakeShared<FJsonValueNumber>(Vector.Z));
			Json->SetArrayField(Elem.Key, Array);
		}
		if (Tag == ERustPropertyTag::Class)
		{
			size_t Ptr = reinterpret_cast<size_t>(Elem.Value.Class.Get());
			Json->SetStringField(Elem.Key, FString::Format(TEXT("{0}"), {Ptr}));
		}
		if (Tag == ERustPropertyTag::Sound)
		{
			size_t Ptr = reinterpret_cast<size_t>(Elem.Value.Sound.Get());
			Json->SetStringField(Elem.Key, FString::Format(TEXT("{0}"), {Ptr}));
		}
	}
	FString OutputString;
	TSharedRef<TJsonWriter<>> Writer = TJsonWriterFactory<>::Create(&OutputString);
	FJsonSerializer::Serialize(Json.ToSharedRef(), Writer);

	return OutputString;
}

#undef LOCTEXT_NAMESPACE
