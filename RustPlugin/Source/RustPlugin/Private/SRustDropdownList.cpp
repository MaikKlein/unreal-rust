// Fill out your copyright notice in the Description page of Project Settings.


#include "SRustDropdownList.h"

#include "RustPlugin.h"
#include "RustUtils.h"
#include "SlateOptMacros.h"
#include "Widgets/Input/SSearchBox.h"
#include "Widgets/Layout/SScrollBox.h"
#include "SListViewSelectorDropdownMenu.h"

BEGIN_SLATE_FUNCTION_BUILD_OPTIMIZATION
#define LOCTEXT_NAMESPACE "RustDropdownList"

void SRustDropdownList::Construct(const FArguments& InArgs)
{
	OnUuidPicked = InArgs._OnUuidPickedDelegate;

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
					.OnGenerateRow(this, &SRustDropdownList::OnGenerateRowForList)
					.OnSelectionChanged(this, &SRustDropdownList::OnClassViewerSelectionChanged);
	SAssignNew(SearchBox, SSearchBox)
		.OnTextChanged(this, &SRustDropdownList::OnFilterTextChanged)
		.HintText(LOCTEXT("ArrayAddElementSearchBoxHint", "Search Elements"));
	ChildSlot[
		SNew(SComboButton)
		//.ButtonContent()[
		//	SNew(STextBlock)
		//	.Text(LOCTEXT("ASD", "Foo"))
		//]
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
	];
}

TSharedRef<ITableRow> SRustDropdownList::OnGenerateRowForList(TSharedPtr<FUuidViewNode> Item,
                                                              const TSharedRef<STableViewBase>& OwnerTable) const
{
	//Create the row
	return
		SNew(STableRow< TSharedPtr<FString> >, OwnerTable)
		.Padding(2.0f)[
			SNew(STextBlock).Text(FText::FromString(*Item.Get()->Name))
		];
}

void SRustDropdownList::OnClassViewerSelectionChanged(TSharedPtr<FUuidViewNode> Item,
                                                      ESelectInfo::Type SelectInfo) const
{
	if (Item.Get() != nullptr)
	{
		FText NewText = FText::FromString(Item.Get()->Name);
		//Selected->SetText(NewText);

		if (SelectInfo != ESelectInfo::OnNavigation)
		{
			OnUuidPicked.ExecuteIfBound(Item.Get());
		}
	}
}

void SRustDropdownList::OnFilterTextChanged(const FText& InNewText)
{
	Items = FilterItems(AllItems, InNewText);
	ListViewWidget->RequestListRefresh();

	if (!Items.IsEmpty())
	{
		ListViewWidget->SetSelection(Items[0], ESelectInfo::OnNavigation);
	}
}

TArray<TSharedPtr<FUuidViewNode>> SRustDropdownList::FilterItems(TArray<TSharedPtr<FUuidViewNode>>& AllItems,
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

#undef LOCTEXT_NAMESPACE
END_SLATE_FUNCTION_BUILD_OPTIMIZATION
