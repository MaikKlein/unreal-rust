#include "FRustDetailCustomization.h"

// MyCustomization.cpp
#include "PropertyEditing.h"
#include "Widgets/Input/SVectorInputBox.h"
#include "RustActor.h"

#define LOCTEXT_NAMESPACE "RustDetailCustomization"

TSharedRef<IDetailCustomization> FRustDetailCustomization::MakeInstance()
{
	return MakeShareable(new FRustDetailCustomization);
}

void FRustDetailCustomization::CustomizeDetails(IDetailLayoutBuilder& DetailBuilder)
{
	TArray<TWeakObjectPtr<UObject>> Objects;
	DetailBuilder.GetObjectsBeingCustomized(Objects);

	UE_LOG(LogTemp, Warning, TEXT("Objects %i"), Objects.Num());
	if (Objects.IsEmpty())
		return;

	TWeakObjectPtr<UEntityComponent> RustActor = Cast<UEntityComponent>(Objects[0]);

	if (RustActor == nullptr)
		return;
	UE_LOG(LogTemp, Warning, TEXT("Is Rust Actor"));


	IDetailCategoryBuilder& RustCategory = DetailBuilder.EditCategory(TEXT("Rust"));

	RustCategory.AddCustomRow(LOCTEXT("RustCategory", "Components")).WholeRowContent()[
		SNew(SVerticalBox)
		+ SVerticalBox::Slot()[
			SNew(SButton)
			.Text(LOCTEXT("RegenerateBtnText", "Regenerate List"))
		]
		+ SVerticalBox::Slot()[
			SNew(SNumericVectorInputBox<FVector::FReal>).bColorAxisLabels(true)
		]
	];
}

#undef LOCTEXT_NAMESPACE
