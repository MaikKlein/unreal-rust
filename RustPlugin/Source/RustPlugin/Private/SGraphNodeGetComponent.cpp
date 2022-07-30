#include "SGraphNodeGetComponent.h"
#include "SListViewSelectorDropdownMenu.h"
#include "Widgets/Layout/SScrollBox.h"

#define LOCTEXT_NAMESPACE "GetComponentRust"

void SGraphNodeGetComponent::Construct(const FArguments& InArgs, UK2Node* Node)
{
	OnUuidPicked = InArgs._OnUuidPickedDelegate;
	this->GraphNode = Node;
	this->SetCursor(EMouseCursor::CardinalCross);
	this->UpdateGraphNode();
	SelectedComponentText = InArgs._SelectedComponentText;
}

void SGraphNodeGetComponent::CreateBelowWidgetControls(TSharedPtr<SVerticalBox> MainBox)
{
	MainBox->AddSlot()
	[
		SNew(SRustDropdownList).OnUuidPickedDelegate(OnUuidPicked)
	];
}
#undef LOCTEXT_NAMESPACE
