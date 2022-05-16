#include "FUuidGraphPanelPinFactory.h"
#include "SGraphPin.h"

TSharedPtr<class SGraphPin> FUuidGraphPanelPinFactory::CreatePin(class UEdGraphPin* InPin) const
{
	//return SNew(SGraphPin, InPin);
	return TSharedPtr<SGraphPin>();
}
