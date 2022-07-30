#pragma once
#include "CoreMinimal.h"
#include "KismetNodes/SGraphNodeK2Default.h"
#include "Widgets/Input/SSearchBox.h"
#include "SRustDropdownList.h"


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
	virtual void CreateBelowWidgetControls(TSharedPtr<SVerticalBox> MainBox) override;
private:
	FOnUuidPicked OnUuidPicked;
	TAttribute<FText> SelectedComponentText;
};
