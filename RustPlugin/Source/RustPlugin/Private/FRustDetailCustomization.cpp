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
	//TSharedRef<IPropertyHandle> Handle = DetailBuilder.GetProperty(
	//	GET_MEMBER_NAME_CHECKED(UEntityComponent, Components));
	TSharedRef<IPropertyHandle> ComponentsHandle = DetailBuilder.GetProperty(
		GET_MEMBER_NAME_CHECKED(UEntityComponent, Components));

	
	IDetailCategoryBuilder& RustCategory = DetailBuilder.EditCategory(TEXT("Rust"));
	auto OnPicked = [Component, &DetailBuilder, ComponentsHandle](FUuidViewNode* Node)
	{
		if (Node == nullptr || Component == nullptr)
			return;

		//{
		//	const FScopedTransaction Transaction(LOCTEXT("AddComponent", "Add Rust component to actor"));

		//	Handle->AsMap()->AddItem();
		//	uint32 NumChildren = 0;
		//	Handle->GetNumChildren(NumChildren);
		//	auto ChildProp = Handle->GetChildHandle(NumChildren - 1);
		//	auto KeyProp = ChildProp->GetKeyHandle();

		//	//KeyProp->NotifyPreChange();
		//	//TArray<void*> RawData;
		//	//KeyProp->AccessRawData(RawData);
		//	//FGuid* Key = static_cast<FGuid*>(RawData[0]);
		//	//*Key = Node->Id;
		//	//KeyProp->NotifyPostChange(EPropertyChangeType::ValueSet);
		//	//KeyProp->NotifyFinishedChangingProperties();

		//	KeyProp->SetValue(Node->Id.ToString());
		//	{
		//		auto DynRust = NewObject<UDynamicRustComponent>(Component.Get()->GetPackage());
		//		DynRust->Initialize(Node->Id, Component.Get()->GetPackage());
		//		ChildProp->SetValue(DynRust);
		//	}
		//}
		{
			ComponentsHandle->AsMap()->AddItem();
			uint32 NumChildren = 0;
			ComponentsHandle->GetNumChildren(NumChildren);
			auto ChildProp = ComponentsHandle->GetChildHandle(NumChildren - 1);
			auto KeyProp = ChildProp->GetKeyHandle();

			KeyProp->SetValue(Node->Id.ToString());
			{
				//auto DynRust = FDynamicRustComponent2();
				//FRustProperty2 Prop2;
				//Prop2.Bar = 42;
				//Prop2.Foo = 42;
				//DynRust.Fields.Add(FString(TEXT("Foo")), Prop2);
				//ChildProp->NotifyPreChange();
				//TArray<void*> RawData;
				//ChildProp->AccessRawData(RawData);
				//FDynamicRustComponent2* Val = static_cast<FDynamicRustComponent2*>(RawData[0]);
				//*Val = DynRust;
				//reinterpret_cast<FDynamicRustComponent2*>(Struct->GetStructMemory())->Initialize(ObjectPropertyHandle);
				//ChildProp->NotifyPostChange(EPropertyChangeType::ValueSet);
				//ChildProp->NotifyFinishedChangingProperties();




				//TSharedRef<FStructOnScope> Struct = MakeShared<FStructOnScope>(FDynamicRustComponent2::StaticStruct());
				//ChildProp->AddChildStructure(Struct);
				FDynamicRustComponent2::Initialize(ChildProp, Node->Id);
			}
		}
		DetailBuilder.ForceRefreshDetails();
	};

	//for (auto& Elem : Component->Components2)
	//{
	//	FString GuidName = Elem.Key;
	//	FGuid Guid;
	//	FGuid::Parse(GuidName, Guid);
	//	auto OnRemoved = FOnComponentRemoved::CreateLambda([Component, &DetailBuilder, GuidName]() -> FReply
	//	{
	//		UE_LOG(LogTemp, Warning, TEXT("Removed clicked"));
	//		const FScopedTransaction Transaction(LOCTEXT("RemoveComponent", "Add Rust component to actor"));
	//		Component->Modify(true);
	//		Component->GetOwner()->Modify(true);
	//		Component->Components.Remove(GuidName);
	//		DetailBuilder.ForceRefreshDetails();
	//		return FReply::Handled();
	//	});
	//	
	//}
	FDynamicRustComponent2::Render(ComponentsHandle, RustCategory,DetailBuilder.GetPropertyUtilities(), FOnComponentRemoved());

	RustCategory.AddCustomRow(LOCTEXT("Picker", "Picker")).WholeRowContent()[
		SNew(SRustDropdownList).OnUuidPickedDelegate(FOnUuidPicked::CreateLambda(OnPicked))
	];
}

#undef LOCTEXT_NAMESPACE
