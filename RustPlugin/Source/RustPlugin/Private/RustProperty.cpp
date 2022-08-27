// Fill out your copyright notice in the Description page of Project Settings.

#include "RustProperty.h"

#include "DetailLayoutBuilder.h"
#include "RustPlugin.h"
#include "RustUtils.h"
#include "DetailWidgetRow.h"
#include "IContentBrowserSingleton.h"
#include "IDetailGroup.h"
#include "EditorWidgets/Public/SAssetDropTarget.h"
#include "GameFramework/GameModeBase.h"
#include "Widgets/Input/SVectorInputBox.h"

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

	Uuid Id = ToUuid(InitGuid);

	auto Reflection = GetRustModule().Plugin.ReflectionData.Types.Find(InitGuid);
	NameProperty->SetValue(Reflection->Name);

	uint32_t NumberOfFields = Reflection->IndexToFieldName.Num();

	for (uint32_t Idx = 0; Idx < NumberOfFields; ++Idx)
	{
		auto& FieldName = *Reflection->IndexToFieldName.Find(Idx);
		auto Type = *Reflection->FieldNameToType.Find(FieldName);

		uint32 FieldPropertyIndex = 0;
		RustPropertyMap->GetNumChildren(FieldPropertyIndex);
		RustPropertyMap->AsMap()->AddItem();

		auto RustPropertyEntry = RustPropertyMap->GetChildHandle(FieldPropertyIndex);
		RustPropertyEntry->GetKeyHandle()->SetValue(FieldName);
		FRustProperty::Initialize(RustPropertyEntry, Type);
	}
}

void FDynamicRustComponent::Render(TSharedRef<IPropertyHandle> MapHandle, IDetailCategoryBuilder& DetailBuilder,
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

		uint32 NumberOfFields = 0;
		FieldsProperty->GetNumChildren(NumberOfFields);

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
}

#undef LOCTEXT_NAMESPACE
