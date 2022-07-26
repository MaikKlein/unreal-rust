#pragma once
#include "CoreMinimal.h"
#include "KismetNodes/SGraphNodeK2Default.h"
#include "Widgets/Input/SSearchBox.h"
#include "SGraphNodeGetComponent.generated.h"

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

class SGraphNodeGetComponent : public SGraphNode
{
public:
	SLATE_BEGIN_ARGS(SGraphNodeGetComponent)
		{
		}

		SLATE_ARGUMENT(FOnUuidPicked, OnUuidPickedDelegate)
		SLATE_ATTRIBUTE( FText, SelectedComponentText );

	SLATE_END_ARGS()

	void Construct(const FArguments& InArgs, UK2Node* Node);
	static TArray<TSharedPtr<FUuidViewNode>> FilterItems(TArray<TSharedPtr<FUuidViewNode>>& AllItems, FText Text);
	void OnFilterTextChanged(const FText& InNewText);
	virtual void CreateBelowWidgetControls(TSharedPtr<SVerticalBox> MainBox) override;
	TArray<TSharedPtr<FUuidViewNode>> Items;
	TArray<TSharedPtr<FUuidViewNode>> AllItems;
	TSharedRef<ITableRow> OnGenerateRowForList(TSharedPtr<FUuidViewNode> Item,
	                                           const TSharedRef<STableViewBase>& OwnerTable) const;
	TSharedPtr<SListView<TSharedPtr<FUuidViewNode>>> ListViewWidget;
	void OnClassViewerSelectionChanged(TSharedPtr<FUuidViewNode> Item, ESelectInfo::Type SelectInfo) const;
	TSharedPtr<SSearchBox> SearchBox;
	TSharedPtr<STextBlock> Selected;
	FOnUuidPicked OnUuidPicked;
private:
	TAttribute<FText> SelectedComponentText;
};
