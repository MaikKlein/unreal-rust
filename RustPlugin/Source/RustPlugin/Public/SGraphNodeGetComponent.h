#pragma once
#include "CoreMinimal.h"
#include "Bindings.h"
#include "KismetNodes/SGraphNodeK2Default.h"
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

	SLATE_END_ARGS()

	void Construct(const FArguments& InArgs, UK2Node* Node);
	virtual void CreateBelowWidgetControls(TSharedPtr<SVerticalBox> MainBox) override;
	TArray<TSharedPtr<FUuidViewNode>> Items;
	TSharedRef<ITableRow> OnGenerateRowForList(TSharedPtr<FUuidViewNode> Item,
	                                           const TSharedRef<STableViewBase>& OwnerTable);
	TSharedPtr<SListView<TSharedPtr<FUuidViewNode>>> ListViewWidget;
	void OnClassViewerSelectionChanged(TSharedPtr<FUuidViewNode> Item, ESelectInfo::Type SelectInfo);
	FOnUuidPicked OnUuidPicked;
};
