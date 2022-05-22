#pragma once
#include "CoreMinimal.h"
#include "KismetNodes/SGraphNodeK2Default.h"

DECLARE_DELEGATE_OneParam( FOnUuidPicked, FString* );
class SGraphNodeGetComponent : public SGraphNode
{
public:
	SLATE_BEGIN_ARGS( SGraphNodeGetComponent )
	{
	}

	SLATE_ARGUMENT( FOnUuidPicked, OnUuidPickedDelegate)

	SLATE_END_ARGS()
	void Construct(const FArguments& InArgs, UK2Node* Node);
	virtual void CreateBelowWidgetControls(TSharedPtr<SVerticalBox> MainBox) override;
	TArray<TSharedPtr<FString>> Items;
	TSharedRef<ITableRow> OnGenerateRowForList(TSharedPtr<FString> Item, const TSharedRef<STableViewBase>& OwnerTable);
	TSharedPtr<SListView<TSharedPtr<FString>>> ListViewWidget;
	void OnClassViewerSelectionChanged( TSharedPtr<FString> Item, ESelectInfo::Type SelectInfo );
	FOnUuidPicked OnUuidPicked;
};
