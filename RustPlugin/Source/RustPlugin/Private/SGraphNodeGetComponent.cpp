#include "SGraphNodeGetComponent.h"
#include "SListViewSelectorDropdownMenu.h"
#include "RustPlugin.h"
#include "RustUtils.h"
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

void SGraphNodeGetComponent::OnFilterTextChanged(const FText& InNewText)
{
	Items = FilterItems(AllItems, InNewText);
	ListViewWidget->RequestListRefresh();

	if (!Items.IsEmpty())
	{
		ListViewWidget->SetSelection(Items[0], ESelectInfo::OnNavigation);
	}
}

TArray<TSharedPtr<FUuidViewNode>> SGraphNodeGetComponent::FilterItems(TArray<TSharedPtr<FUuidViewNode>>& AllItems,
                                                                      FText Text)
{
	FString TrimmedFilterString = FText::TrimPrecedingAndTrailing(Text).ToString();
	if (TrimmedFilterString.IsEmpty())
	{
		return AllItems;
	}
	TArray<TSharedPtr<FUuidViewNode>> Result;

	for (auto& Item : AllItems)
	{
		// TODO: We can do a better search filter than that. Maybe check how the asset browser filters things?
		if (Item->Name.Contains(TrimmedFilterString))
		{
			Result.Add(Item);
		}
	}
	return Result;
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
			AllItems.Add(MakeShareable(Node));
		}
	}

	Items = AllItems;

	SAssignNew(ListViewWidget, SListView<TSharedPtr<FUuidViewNode>>)
					.ItemHeight(24)
					.ListItemsSource(&Items) //The Items array is the source of this listview
					.SelectionMode(ESelectionMode::Single)
					.OnGenerateRow(this, &SGraphNodeGetComponent::OnGenerateRowForList)
					.OnSelectionChanged(this, &SGraphNodeGetComponent::OnClassViewerSelectionChanged);
	SAssignNew(SearchBox, SSearchBox)
		.OnTextChanged(this, &SGraphNodeGetComponent::OnFilterTextChanged)
		.HintText(LOCTEXT("ArrayAddElementSearchBoxHint", "Search Elements"));
	MainBox->AddSlot()
	[
		SNew(SScrollBox)
		+ SScrollBox::Slot()
		[
			SNew(SComboButton)
			.ButtonContent()[
				SAssignNew(Selected, STextBlock)
				.Text(SelectedComponentText)
			]
			.MenuContent()[
				SNew(SListViewSelectorDropdownMenu<TSharedPtr<FUuidViewNode>>, SearchBox, ListViewWidget)
				[
					SNew(SVerticalBox)
					+ SVerticalBox::Slot()
					  .AutoHeight()
					  .Padding(4.f, 4.f, 4.f, 4.f)
					[
						SearchBox.ToSharedRef()
					]
					+ SVerticalBox::Slot()
					  .AutoHeight()
					  .Padding(4.f, 4.f, 4.f, 4.f)
					[
						ListViewWidget.ToSharedRef()
					]
				]
			]
		]
	];
}

TSharedRef<ITableRow> SGraphNodeGetComponent::OnGenerateRowForList(TSharedPtr<FUuidViewNode> Item,
                                                                   const TSharedRef<STableViewBase>& OwnerTable) const
{
	//Create the row
	return
		SNew(STableRow< TSharedPtr<FString> >, OwnerTable)
		.Padding(2.0f)[
			SNew(STextBlock).Text(FText::FromString(*Item.Get()->Name))
		];
}

void SGraphNodeGetComponent::OnClassViewerSelectionChanged(TSharedPtr<FUuidViewNode> Item, ESelectInfo::Type SelectInfo) const
{
	if (Item.Get() != nullptr)
	{
		FText NewText = FText::FromString(Item.Get()->Name);
		Selected->SetText(NewText);

		if (SelectInfo != ESelectInfo::OnNavigation)
		{
			OnUuidPicked.ExecuteIfBound(Item.Get());
		}
	}
}
#undef LOCTEXT_NAMESPACE
