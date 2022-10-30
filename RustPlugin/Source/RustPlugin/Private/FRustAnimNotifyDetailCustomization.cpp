#include "FRustAnimNotifyDetailCustomization.h"

// MyCustomization.cpp
#include "AnimNotify_RustEvent.h"
#include "DetailLayoutBuilder.h"
#include "DetailWidgetRow.h"
#include "Widgets/Input/SVectorInputBox.h"
#include "RustActor.h"
#include "RustProperty.h"
#include "SRustDropdownList.h"

#define LOCTEXT_NAMESPACE "RustDetailCustomization"

TSharedRef<IDetailCustomization> FRustAnimNotifyDetailCustomization::MakeInstance()
{
	return MakeShareable(new FRustAnimNotifyDetailCustomization);
}

void FRustAnimNotifyDetailCustomization::CustomizeDetails(IDetailLayoutBuilder& DetailBuilder)
{
	UE_LOG(LogTemp, Warning, TEXT("Custom"));
	TArray<TWeakObjectPtr<UObject>> Objects;
	DetailBuilder.GetObjectsBeingCustomized(Objects);

	if (Objects.IsEmpty())
		return;

	TWeakObjectPtr<UAnimNotify_RustEvent> RustEvent = Cast<UAnimNotify_RustEvent>(Objects[0]);

	TSharedRef<IPropertyHandle> GuidHandle = DetailBuilder.GetProperty(
		GET_MEMBER_NAME_CHECKED(UAnimNotify_RustEvent, Guid));
	TSharedRef<IPropertyHandle> EventHandle = DetailBuilder.GetProperty(
		GET_MEMBER_NAME_CHECKED(UAnimNotify_RustEvent, Event));
	
	// Don't show the components map in the editor. This should only be edited through the custom ui below.
	DetailBuilder.HideProperty(GuidHandle);
	DetailBuilder.HideProperty(EventHandle);
	{

		FString GuidName;
		GuidHandle->GetValue(GuidName);

		FGuid Guid;
		FGuid::Parse(GuidName, Guid);
		
		//RustEvent->Event.Reload(EventHandle, Guid);
	}


	IDetailCategoryBuilder& RustCategory = DetailBuilder.EditCategory(TEXT("Rust"));
	FDynamicRustComponent::Render(EventHandle, RustCategory, DetailBuilder);

	auto OnPicked = [&DetailBuilder, GuidHandle, EventHandle](FUuidViewNode* Node)
	{
		if (Node == nullptr)
			return;

		{
			GuidHandle->SetValue(Node->Id.ToString());
			FDynamicRustComponent::Initialize(EventHandle, Node->Id);
		}
		DetailBuilder.ForceRefreshDetails();
	};

	RustCategory.AddCustomRow(LOCTEXT("Picker", "Picker")).WholeRowContent()[
		SNew(SRustDropdownList).OnlyShowEditorComponents(true).OnUuidPickedDelegate(
			FOnUuidPicked::CreateLambda(OnPicked))
	];
}

#undef LOCTEXT_NAMESPACE
