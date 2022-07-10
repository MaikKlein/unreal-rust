#include "SGraphNodeGetComponent.h"

#include "Utils.h"
#include "RustPlugin.h"
#include "Widgets/Layout/SScrollBox.h"

#define LOCTEXT_NAMESPACE "GetComponentRust"

void SGraphNodeGetComponent::Construct(const FArguments& InArgs, UK2Node* Node)
{
	OnUuidPicked = InArgs._OnUuidPickedDelegate;
	this->GraphNode = Node;
	this->SetCursor(EMouseCursor::CardinalCross);
	this->UpdateGraphNode();
}

void SGraphNodeGetComponent::CreateBelowWidgetControls(TSharedPtr<SVerticalBox> MainBox)
{
	for (Uuid Id : GetModule().Plugin.Uuids)
	{
		const char* Name;
		uintptr_t Len = 0;
		if (GetModule().Plugin.Rust.reflection_fns.get_type_name(Id, &Name, &Len))
		{
			FUuidViewNode* Node = new FUuidViewNode();
			Node->Name = FString(Len, UTF8_TO_TCHAR(Name));
			Node->Id = ToFGuid(Id);
			Items.Add(MakeShareable(Node));
		}
	}

	MainBox->AddSlot()[
		SNew(SScrollBox)
		+ SScrollBox::Slot()[
			SAssignNew(ListViewWidget, SListView<TSharedPtr<FUuidViewNode>>)
					.ItemHeight(24)
					.ListItemsSource(&Items) //The Items array is the source of this listview
					.SelectionMode(ESelectionMode::Single)
					.OnGenerateRow(this, &SGraphNodeGetComponent::OnGenerateRowForList)
					.OnSelectionChanged(this, &SGraphNodeGetComponent::OnClassViewerSelectionChanged)
		]
	];
}

TSharedRef<ITableRow> SGraphNodeGetComponent::OnGenerateRowForList(TSharedPtr<FUuidViewNode> Item,
                                                                   const TSharedRef<STableViewBase>& OwnerTable)
{
	//Create the row
	return
		SNew(STableRow< TSharedPtr<FString> >, OwnerTable)
		.Padding(2.0f)[

			SNew(STextBlock).Text(FText::FromString(*Item.Get()->Name))
		];
}

void SGraphNodeGetComponent::OnClassViewerSelectionChanged(TSharedPtr<FUuidViewNode> Item, ESelectInfo::Type SelectInfo)
{
	OnUuidPicked.ExecuteIfBound(Item.Get());
}
#undef LOCTEXT
