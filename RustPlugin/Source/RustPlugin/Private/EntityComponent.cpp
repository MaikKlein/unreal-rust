// Fill out your copyright notice in the Description page of Project Settings.


#include "EntityComponent.h"

#include "DetailWidgetRow.h"
#include "RustPlugin.h"
#include "RustUtils.h"
#include "Widgets/Input/SRotatorInputBox.h"
#include "Widgets/Input/SVectorInputBox.h"
#include "Widgets/Input/NumericTypeInterface.h"
#include "Widgets/Input/NumericUnitTypeInterface.inl"

#define LOCTEXT_NAMESPACE "RustDetailCustomization"

void UDynamicRustComponent::Render(IDetailCategoryBuilder& DetailBuilder, FOnComponentRemoved OnComponentRemoved)
{
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
	for (auto& Elem : Fields)
	{
		auto BoolProp = Cast<URustPropertyBool>(Elem.Value);
		auto VectorProp = Cast<URustPropertyVector>(Elem.Value);
		auto QuatProp = Cast<URustPropertyQuaternion>(Elem.Value);
		auto FloatProp = Cast<URustPropertyFloat>(Elem.Value);

		if (Elem.Value == nullptr)
			UE_LOG(LogTemp, Warning, TEXT("Noooopppppe"));

		TSharedRef<SWidget> Widget = SNew(SCheckBox);

		auto& W = DetailBuilder.AddCustomRow(RowText).NameContent()
		[
			SNew(STextBlock).Text(FText::FromString(Elem.Key))
		];
		if (BoolProp != nullptr)
		{
			auto Value = [=]() -> ECheckBoxState
			{
				if (BoolProp->Data)
				{
					return ECheckBoxState::Checked;
				}
				else
				{
					return ECheckBoxState::Unchecked;
				}
			};
			auto OnChanged = [=](ECheckBoxState Changed)
			{
				if (Changed == ECheckBoxState::Checked)
				{
					BoolProp->Data = true;
					return;
				}
				BoolProp->Data = false;
			};
			W.ValueContent()[
				SNew(SCheckBox).IsChecked_Lambda(Value).OnCheckStateChanged_Lambda(OnChanged)
			];
		}
		if (FloatProp != nullptr)
		{
			auto OnChanged = [=](float Val)
			{
				FloatProp->Data = Val;
			};
			auto Value = [=]() -> float
			{
				return FloatProp->Data;
			};
			W.ValueContent()[
				SNew(SNumericEntryBox<float>).OnValueChanged_Lambda(OnChanged).Value_Lambda(Value)
			];
		}
		if (QuatProp != nullptr)
		{
			auto Roll = [=]() -> TOptional<FRotator::FReal>
			{
				return QuatProp->Data.Rotator().Roll;
			};
			auto Yaw = [=]() -> TOptional<FRotator::FReal>
			{
				return QuatProp->Data.Rotator().Yaw;
			};
			auto Pitch = [=]() -> TOptional<FRotator::FReal>
			{
				return QuatProp->Data.Rotator().Pitch;
			};
			auto OnRollChanged = [=](FRotator::FReal Val)
			{
				FRotator R = QuatProp->Data.Rotator();
				R.Roll = Val;
				QuatProp->Data = R.Quaternion();
			};
			auto OnYawChanged = [=](FRotator::FReal Val)
			{
				FRotator R = QuatProp->Data.Rotator();
				R.Yaw = Val;
				QuatProp->Data = R.Quaternion();
			};
			auto OnPitchChanged = [=](FRotator::FReal Val)
			{
				FRotator R = QuatProp->Data.Rotator();
				R.Pitch = Val;
				QuatProp->Data = R.Quaternion();
			};
			TSharedPtr<TNumericUnitTypeInterface<FRotator::FReal>> DegreesTypeInterface =
				MakeShareable(new TNumericUnitTypeInterface<FRotator::FReal>(EUnit::Degrees));
			W.ValueContent()
			 .MinDesiredWidth(125.0f * 3.0f)
			 .MaxDesiredWidth(125.0f * 3.0f)
			 .VAlign(VAlign_Center)
			[
				SNew(SNumericRotatorInputBox<FRotator::FReal>)
				.TypeInterface(DegreesTypeInterface)
				.AllowSpin(true).bColorAxisLabels(true)
				.Pitch_Lambda(Pitch)
				.Yaw_Lambda(Yaw)
				.Roll_Lambda(Roll)
				.OnRollChanged_Lambda(OnRollChanged)
				.OnPitchChanged_Lambda(OnPitchChanged)
				.OnYawChanged_Lambda(OnYawChanged)
			];
		}
		if (VectorProp != nullptr)
		{
			auto SetVector = [=](FVector V)
			{
				VectorProp->Data = V;
				UE_LOG(LogTemp, Warning, TEXT("Changed %s"), *VectorProp->Data.ToString());
			};
			auto SetVectorCommitted = [=](FVector V, ETextCommit::Type Ty)
			{
				VectorProp->Data = V;
				UE_LOG(LogTemp, Warning, TEXT("Commit %s"), *VectorProp->Data.ToString());
			};
			auto GetValue = [=]() -> TOptional<FVector>
			{
				return TOptional(VectorProp->Data);
			};
			auto OnXChanged = [=](double Val)
			{
				VectorProp->Data.X = Val;
			};
			auto OnYChanged = [=](double Val)
			{
				VectorProp->Data.Y = Val;
			};
			auto OnZChanged = [=](double Val)
			{
				VectorProp->Data.Z = Val;
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
	}
}

// Sets default values for this component's properties
UEntityComponent::UEntityComponent()
{
	PrimaryComponentTick.bCanEverTick = false;
}

FEntity UEntityComponent::GetEntity()
{
	return Id;
}


void UDynamicRustComponent::Initialize(FGuid Guid, UObject* Owner)
{
	auto Fns = &GetModule().Plugin.Rust.reflection_fns;
	Uuid Id = ToUuid(Guid);

	const char* TypeNamePtr = nullptr;
	uintptr_t TypeNameLen = 0;
	Fns->get_type_name(Id, &TypeNamePtr, &TypeNameLen);
	Name = FString(TypeNameLen, UTF8_TO_TCHAR(TypeNamePtr));

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


		if (Type == ReflectionType::Vector3)
		{
			Fields.Add(FieldName, NewObject<URustPropertyVector>(Owner));
		}

		if (Type == ReflectionType::Bool)
		{
			Fields.Add(FieldName, NewObject<URustPropertyBool>(Owner));
		}
		if (Type == ReflectionType::Float)
		{
			Fields.Add(FieldName, NewObject<URustPropertyFloat>(Owner));
		}
		if (Type == ReflectionType::Quaternion)
		{
			Fields.Add(FieldName, NewObject<URustPropertyQuaternion>(Owner));
		}
	}
}

#undef LOCTEXT_NAMESPACE
