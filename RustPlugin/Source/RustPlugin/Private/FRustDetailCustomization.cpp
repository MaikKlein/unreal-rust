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

	UE_LOG(LogTemp, Warning, TEXT("Objects %i"), Objects.Num());
	if (Objects.IsEmpty())
		return;

	for (UObject* Owner = Objects[0].Get(); Owner; Owner = Owner->GetOuter())
	{
		if (Owner == nullptr)
			break;
		UE_LOG(LogTemp, Warning, TEXT("%s"), *Owner->GetClass()->GetName());
	}
	TWeakObjectPtr<UEntityComponent> Component = Cast<UEntityComponent>(Objects[0]);

	if (Component == nullptr)
		return;
	UE_LOG(LogTemp, Warning, TEXT("Is Rust Actor"));


	IDetailCategoryBuilder& RustCategory = DetailBuilder.EditCategory(TEXT("Rust"));
	auto OnPicked = [Component](FUuidViewNode* Node)
	{
		if (Node == nullptr || Component == nullptr)
			return;

		auto DynRust = NewObject<UDynamicRustComponent>(Component->GetPackage());
		DynRust->Initialize(Node->Id);
		Component->Components.Add(Node->Id, DynRust);
	};
	auto Box = SNew(SVerticalBox);
	for (auto& Elem : Component->Components)
	{
		if (Elem.Value == nullptr)
			continue;
		Elem.Value->Render(Box);
	}
	Box->AddSlot()[
		SNew(SRustDropdownList).OnUuidPickedDelegate(FOnUuidPicked::CreateLambda(OnPicked))
	];
	RustCategory.AddCustomRow(LOCTEXT("RustCategory", "Components")).WholeRowContent()[
		Box
	];
}

#undef LOCTEXT_NAMESPACE
