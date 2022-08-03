#include "FRustDetailCustomization.h"

// MyCustomization.cpp
#include "PropertyEditing.h"
#include "Widgets/Input/SVectorInputBox.h"
#include "RustActor.h"
#include "SRustDropdownList.h"

#define LOCTEXT_NAMESPACE "RustDetailCustomization"

TSharedRef<IDetailCustomization> FRustDetailCustomization::MakeInstance()
{
	return MakeShareable(new FRustDetailCustomization);
}

void FRustDetailCustomization::CustomizeDetails(IDetailLayoutBuilder& DetailBuilder)
{
	TArray<TWeakObjectPtr<UObject>> Objects;
	DetailBuilder.GetObjectsBeingCustomized(Objects);

	if (Objects.IsEmpty())
		return;

	TWeakObjectPtr<UEntityComponent> Component = Cast<UEntityComponent>(Objects[0]);

	IDetailCategoryBuilder& RustCategory = DetailBuilder.EditCategory(TEXT("Rust"));
	auto OnPicked = [Component, &DetailBuilder](FUuidViewNode* Node)
	{
		if (Node == nullptr || Component == nullptr)
			return;

		auto DynRust = NewObject<UDynamicRustComponent>(Component->GetPackage());
		DynRust->Initialize(Node->Id);
		Component->Components.Add(Node->Id, DynRust);
		DetailBuilder.ForceRefreshDetails();
	};
	
	for (auto& Elem : Component->Components)
	{
		if (Elem.Value == nullptr)
			continue;

		FGuid Guid = Elem.Key;
		auto OnRemoved = FOnComponentRemoved::CreateLambda([Component, &DetailBuilder, Guid]() -> FReply
		{
			UE_LOG(LogTemp, Warning, TEXT("Removed clicked"));
			Component->Components.Remove(Guid);
			DetailBuilder.ForceRefreshDetails();
			return FReply::Handled();
		});
		Elem.Value->Render(RustCategory, OnRemoved);
	}

	RustCategory.AddCustomRow(LOCTEXT("Picker", "Picker")).WholeRowContent()[
		SNew(SRustDropdownList).OnUuidPickedDelegate(FOnUuidPicked::CreateLambda(OnPicked))
	];
}

#undef LOCTEXT_NAMESPACE
