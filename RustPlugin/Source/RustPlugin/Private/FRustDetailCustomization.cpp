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
	auto Handle = DetailBuilder.GetProperty(GET_MEMBER_NAME_CHECKED(UEntityComponent, Components));
	auto IntMap = DetailBuilder.GetProperty(GET_MEMBER_NAME_CHECKED(UEntityComponent, IntMap));

	IDetailCategoryBuilder& RustCategory = DetailBuilder.EditCategory(TEXT("Rust"));
	auto OnPicked = [Component, &DetailBuilder, Handle, IntMap](FUuidViewNode* Node)
	{
		if (Node == nullptr || Component == nullptr)
			return;

		//auto DynRust = NewObject<UDynamicRustComponent>(Component->GetPackage());
		//const FScopedTransaction Transaction(LOCTEXT("AddComponent", "Add Rust component to actor"));
		//{
		//	Handle->NotifyPreChange();
		//	Handle->AsMap()->AddItem();
		//	uint32 NumChildren = 0;
		//	Handle->GetNumChildren(NumChildren);
		//	auto ChildProp = Handle->GetChildHandle(NumChildren - 1);
		//	auto KeyProp = ChildProp->GetKeyHandle();

		//	KeyProp->NotifyPreChange();
		//	TArray<void*> RawData;
		//	KeyProp->AccessRawData(RawData);
		//	FGuid* Key = static_cast<FGuid*>(RawData[0]);
		//	*Key = Node->Id;
		//	KeyProp->NotifyPostChange(EPropertyChangeType::ValueSet);
		//	KeyProp->NotifyFinishedChangingProperties();

		//	auto DynRust = NewObject<UDynamicRustComponent>(Component.Get()->GetPackage());
		//	DynRust->Initialize(Node->Id, DynRust);

		//	ChildProp->SetValue(DynRust);
		//	Handle->NotifyPostChange(EPropertyChangeType::ValueSet);
		//	Handle->NotifyFinishedChangingProperties();
		//}
		{
			IntMap->AsMap()->AddItem();
			uint32 NumChildren = 0;
			IntMap->GetNumChildren(NumChildren);
			auto ChildProp = IntMap->GetChildHandle(NumChildren - 1);
			auto KeyProp = ChildProp->GetKeyHandle();

			//GEditor->BeginTransaction(
			//	FText::Format(LOCTEXT("SetPropertyValue", "Set {0}"), KeyProp->GetPropertyDisplayName()));
			//const FScopedTransaction IntTransaction(LOCTEXT("AddInt", "int"));
			//KeyProp->NotifyPreChange();
			//TArray<void*> RawData;
			//KeyProp->AccessRawData(RawData);
			//int32* Key = static_cast<int32*>(RawData[0]);
			//*Key = NumChildren;
			//KeyProp->NotifyPostChange(EPropertyChangeType::ValueSet);
			//GEditor->EndTransaction();
			//KeyProp->NotifyFinishedChangingProperties();
			KeyProp->SetValue((int32)NumChildren);
			auto DynRust = NewObject<UDynamicRustComponent>(Component.Get()->GetPackage());
			DynRust->Initialize(Node->Id, DynRust);
			ChildProp->SetValue(DynRust);
		}
		//Component->Modify(true);
		//auto DynRust = NewObject<UDynamicRustComponent>(Component.Get(), NAME_None, RF_Transactional);
		//DynRust->Initialize(Node->Id, DynRust);
		//Component->Components.Add(Node->Id, DynRust);
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
			const FScopedTransaction Transaction(LOCTEXT("RemoveComponent", "Add Rust component to actor"));
			Component->Modify(true);
			Component->GetOwner()->Modify(true);
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
