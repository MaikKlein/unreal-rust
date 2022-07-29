// Fill out your copyright notice in the Description page of Project Settings.

#pragma once

#include "CoreMinimal.h"
#include "Widgets/SCompoundWidget.h"
#include "SRustDropdownList.generated.h"

USTRUCT()
struct FUuidViewNode
{
	GENERATED_BODY()
	UPROPERTY()
	FString Name;
	UPROPERTY()
	FGuid Id;
};

DECLARE_DELEGATE_OneParam(FOnUuidPicked, FUuidViewNode*);

class RUSTPLUGIN_API SRustDropdownList : public SCompoundWidget
{
	TArray<TSharedPtr<FUuidViewNode>> Items;
	TArray<TSharedPtr<FUuidViewNode>> AllItems;

	TSharedPtr<SListView<TSharedPtr<FUuidViewNode>>> ListViewWidget;
	TSharedPtr<SSearchBox> SearchBox;
	FOnUuidPicked OnUuidPicked;
public:
	SLATE_BEGIN_ARGS(SRustDropdownList)
		{
		}

		SLATE_ARGUMENT(FOnUuidPicked, OnUuidPickedDelegate)

	SLATE_END_ARGS()

	void Construct(const FArguments& InArgs);
	static TArray<TSharedPtr<FUuidViewNode>> FilterItems(TArray<TSharedPtr<FUuidViewNode>>& AllItems, FText Text);
	
	TSharedRef<ITableRow> OnGenerateRowForList(TSharedPtr<FUuidViewNode> Item,
	                                           const TSharedRef<STableViewBase>& OwnerTable) const;
	void OnClassViewerSelectionChanged(TSharedPtr<FUuidViewNode> Item, ESelectInfo::Type SelectInfo) const;
	void OnFilterTextChanged(const FText& InNewText);
};
