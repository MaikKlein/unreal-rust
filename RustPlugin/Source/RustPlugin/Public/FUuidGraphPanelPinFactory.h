#pragma once

#include "CoreMinimal.h"
#include "EdGraphUtilities.h"

class SGraphPin;

class FUuidGraphPanelPinFactory: public FGraphPanelPinFactory
{
	virtual TSharedPtr<class SGraphPin> CreatePin(class UEdGraphPin* InPin) const override;
};
