#include "FRustDetailCustomization.h"

// MyCustomization.cpp
#include "PropertyEditing.h"
#include "Widgets/Input/SVectorInputBox.h"
#include "RustActor.h"
#include "RustProperty.h"
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

	TSharedRef<IPropertyHandle> ComponentsHandle = DetailBuilder.GetProperty(
		GET_MEMBER_NAME_CHECKED(UEntityComponent, Components));
	// Don't show the components map in the editor. This should only be edited through the custom ui below.
	DetailBuilder.HideProperty(ComponentsHandle);
	{
		uint32 NumChildren = 0;
		ComponentsHandle->GetNumChildren(NumChildren);

		for (uint32 ComponentIdx = 0; ComponentIdx < NumChildren; ++ComponentIdx)
		{
			auto ChildProp = ComponentsHandle->GetChildHandle(ComponentIdx);
			auto KeyProp = ChildProp->GetKeyHandle();

			FString GuidName;
			KeyProp->GetValue(GuidName);

			FGuid Guid;
			FGuid::Parse(GuidName, Guid);

			const auto RustComponent = Component->Components.Find(GuidName);
			if (RustComponent != nullptr)
			{
				RustComponent->Reload(ChildProp, Guid);
			}
		}
	}


	IDetailCategoryBuilder& RustCategory = DetailBuilder.EditCategory(TEXT("Rust"));
	FDynamicRustComponent::Render(ComponentsHandle, RustCategory, DetailBuilder);

	auto OnPicked = [Component, &DetailBuilder, ComponentsHandle](FUuidViewNode* Node)
	{
		if (Node == nullptr || Component == nullptr)
			return;

		if (Component.Get()->Components.Contains(Node->Id.ToString()))
		{
			return;
		}

		{
			uint32 Index = 0;
			ComponentsHandle->GetNumChildren(Index);
			ComponentsHandle->AsMap()->AddItem();
			auto ChildProp = ComponentsHandle->GetChildHandle(Index);
			auto KeyProp = ChildProp->GetKeyHandle();

			KeyProp->SetValue(Node->Id.ToString());
			FDynamicRustComponent::Initialize(ChildProp, Node->Id);
		}
		DetailBuilder.ForceRefreshDetails();
	};

	RustCategory.AddCustomRow(LOCTEXT("Picker", "Picker")).WholeRowContent()[
		SNew(SRustDropdownList).OnlyShowEditorComponents(true).OnUuidPickedDelegate(
			FOnUuidPicked::CreateLambda(OnPicked))
	];
}

#undef LOCTEXT_NAMESPACE
