// Fill out your copyright notice in the Description page of Project Settings.


#include "EntityComponent.h"

#include "DetailWidgetRow.h"
#include "RustPlugin.h"
#include "RustUtils.h"
#include "Widgets/Input/SRotatorInputBox.h"
#include "Widgets/Input/SVectorInputBox.h"

void UDynamicRustComponent::Render(TSharedRef<SVerticalBox> Box)
{
	Box->AddSlot()[
		SNew(SBorder)
				.BorderBackgroundColor(FSlateColor(FColor(0.0, 0.0, 0.0, 1.0)))
				.Padding(0)
		[
			SNew(SVerticalBox)
			+ SVerticalBox::Slot()[
				SNew(SHorizontalBox) +
				SHorizontalBox::Slot()[
					SNew(STextBlock).Text(FText::FromString(Name))
				]
			]]];
	for (auto& Elem : Fields)
	{
		auto BoolProp = Cast<URustPropertyBool>(Elem.Value);
		auto VectorProp = Cast<URustPropertyVector>(Elem.Value);
		auto QuatProp = Cast<URustPropertyQuaternion>(Elem.Value);
		auto FloatProp = Cast<URustPropertyFloat>(Elem.Value);

		if (Elem.Value == nullptr)
			UE_LOG(LogTemp, Warning, TEXT("Noooopppppe"));

		TSharedRef<SWidget> Widget = SNew(SCheckBox);
		if (BoolProp != nullptr)
		{
			Widget = SNew(SCheckBox);
		}
		if (FloatProp != nullptr)
		{
			Widget = SNew(SNumericEntryBox<float>);
		}
		if (QuatProp != nullptr)
		{
			Widget = SNew(SNumericRotatorInputBox<float>);
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
			Widget = SNew(SNumericVectorInputBox<FVector::FReal>)
			.Vector_Lambda(GetValue)
			.OnVectorChanged_Lambda(SetVector)
			.OnXChanged_Lambda([=](double Val)
			                                                     {
				                                                     VectorProp->Data.X = Val;
				                                                     UE_LOG(LogTemp, Warning, TEXT("Commit X %s"),
				                                                            *VectorProp->Data.ToString());
			                                                     })
			.OnVectorCommitted_Lambda(SetVectorCommitted)
			.bColorAxisLabels(true);
		}
		Box->AddSlot()[
			SNew(SBorder)
				.BorderImage(FAppStyle::Get().GetBrush("DetailsView.CategoryMiddle"))
				.BorderBackgroundColor(FSlateColor(FColor(0.0, 0.0, 0.0, 1.0)))
				.Padding(0)
			[
				SNew(SHorizontalBox) +
				SHorizontalBox::Slot()[
					SNew(SSplitter)
					+ SSplitter::Slot().Value(0.3f)[
						SNew(SHorizontalBox) +
						SHorizontalBox::Slot()[
							SNew(STextBlock).Text(FText::FromString(Elem.Key))
						]
					]
					+ SSplitter::Slot().Value(0.7f)[
						SNew(SHorizontalBox) +
						SHorizontalBox::Slot()[
							Widget
						]
					]
				]
			]
		];
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


void UDynamicRustComponent::Initialize(FGuid Guid)
{
	auto Fns = &GetModule().Plugin.Rust.reflection_fns;

	uint32_t NumberOfFields = 0;
	Uuid Id = ToUuid(Guid);
	Fns->number_of_fields(Id, &NumberOfFields);

	for (uint32_t Idx = 0; Idx < NumberOfFields; ++Idx)
	{
		const char* TypeNamePtr = nullptr;
		uintptr_t TypeNameLen = 0;
		Fns->get_type_name(Id, &TypeNamePtr, &TypeNameLen);
		Name = FString(TypeNameLen, UTF8_TO_TCHAR(TypeNamePtr));


		const char* NamePtr = nullptr;
		uintptr_t Len = 0;
		Fns->get_field_name(Id, Idx, &NamePtr, &Len);
		FString FieldName = FString(Len, UTF8_TO_TCHAR(NamePtr));

		ReflectionType Type;
		Fns->get_field_type(Id, Idx, &Type);

		if (Type == ReflectionType::Vector3)
		{
			Fields.Add(FieldName, NewObject<URustPropertyVector>(GetPackage()));
		}

		if (Type == ReflectionType::Bool)
		{
			Fields.Add(FieldName, NewObject<URustPropertyBool>(GetPackage()));
		}
		if (Type == ReflectionType::Float)
		{
			Fields.Add(FieldName, NewObject<URustPropertyFloat>(GetPackage()));
		}
		if (Type == ReflectionType::Quaternion)
		{
			Fields.Add(FieldName, NewObject<URustPropertyQuaternion>(GetPackage()));
		}
	}
}

// Called every frame
void UEntityComponent::TickComponent(float DeltaTime, ELevelTick TickType,
                                     FActorComponentTickFunction* ThisTickFunction)
{
}
