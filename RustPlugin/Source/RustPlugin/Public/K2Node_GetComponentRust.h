#pragma once

#include "K2Node.h"
#include "SGraphNodeGetComponent.h"

#include "K2Node_GetComponentRust.generated.h"

UCLASS()
class UK2Node_GetComponentRust : public UK2Node
{
	GENERATED_BODY()

	virtual void AllocateDefaultPins() override;
	virtual void ExpandNode(class FKismetCompilerContext &CompilerContext, UEdGraph *SourceGraph) override;
	virtual void GetMenuActions(FBlueprintActionDatabaseRegistrar &ActionRegistrar) const override;
	virtual FText GetNodeTitle( ENodeTitleType::Type TitleType ) const override;
	virtual TSharedPtr<SGraphNode> CreateVisualWidget() override;
	void OnUuidPicked(FUuidViewNode* Name);
	UPROPERTY()
	FUuidViewNode SelectedNode;
	void BreakAllOutputPins();
	UPROPERTY()
	TArray<FGuid> IndexPins;
};