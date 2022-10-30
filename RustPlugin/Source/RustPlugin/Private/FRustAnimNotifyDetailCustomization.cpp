#include "FRustAnimNotifyDetailCustomization.h"

// MyCustomization.cpp
#include "AnimNotify_RustEvent.h"
#include "DetailLayoutBuilder.h"
#include "DetailWidgetRow.h"
#include "IDetailChildrenBuilder.h"
#include "IPropertyUtilities.h"
#include "Widgets/Input/SVectorInputBox.h"
#include "RustActor.h"
#include "RustProperty.h"
#include "SRustDropdownList.h"

#define LOCTEXT_NAMESPACE "RustDetailCustomization"

TSharedRef<IPropertyTypeCustomization> FRustAnimNotifyDetailCustomization::MakeInstance()
{
	return MakeShareable(new FRustAnimNotifyDetailCustomization);
}

void FRustAnimNotifyDetailCustomization::CustomizeHeader(TSharedRef<IPropertyHandle> PropertyHandle,
                                                         FDetailWidgetRow& HeaderRow,
                                                         IPropertyTypeCustomizationUtils& CustomizationUtils)
{
	UE_LOG(LogTemp, Warning, TEXT("Custom Header"));
	TSharedPtr<IPropertyHandle> GuidHandle = PropertyHandle->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FRustEvent, Guid));

	UE_LOG(LogTemp, Warning, TEXT("C Header: %i"), GuidHandle.IsValid());
	UE_LOG(LogTemp, Warning, TEXT("Custom Header %s"), *PropertyHandle->GeneratePathToProperty());
}

void FRustAnimNotifyDetailCustomization::CustomizeChildren(TSharedRef<IPropertyHandle> PropertyHandle,
                                                           IDetailChildrenBuilder& ChildBuilder,
                                                           IPropertyTypeCustomizationUtils& CustomizationUtils)
{
	TSharedPtr<IPropertyUtilities> Utilities = CustomizationUtils.GetPropertyUtilities();


	TSharedPtr<IPropertyHandle> GuidHandle = PropertyHandle->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FRustEvent, Guid));

	TSharedPtr<IPropertyHandle> EventHandle = PropertyHandle->GetChildHandle(
		GET_MEMBER_NAME_CHECKED(FRustEvent, Event));
	UE_LOG(LogTemp, Warning, TEXT("Header: %i"), GuidHandle.IsValid());

	// Don't show the components map in the editor. This should only be edited through the custom ui below.
	//DetailBuilder.HideProperty(GuidHandle);
	//DetailBuilder.HideProperty(EventHandle);
	{
		FString GuidName;
		GuidHandle->GetValue(GuidName);

		FGuid Guid;
		FGuid::Parse(GuidName, Guid);

		//RustEvent->Event.Reload(EventHandle, Guid);
	}


	FDynamicRustComponent::Render(EventHandle, ChildBuilder);

	auto OnPicked = [Utilities, GuidHandle, EventHandle](FUuidViewNode* Node)
	{
		if (Node == nullptr)
			return;

		{
			GuidHandle->SetValue(Node->Id.ToString());
			FDynamicRustComponent::Initialize(EventHandle, Node->Id);
		}
		Utilities->ForceRefresh();
	};

	ChildBuilder.AddCustomRow(LOCTEXT("Picker", "Picker")).WholeRowContent()[
		SNew(SRustDropdownList).OnlyShowEditorComponents(true).OnUuidPickedDelegate(
			FOnUuidPicked::CreateLambda(OnPicked))
	];
}

#undef LOCTEXT_NAMESPACE
