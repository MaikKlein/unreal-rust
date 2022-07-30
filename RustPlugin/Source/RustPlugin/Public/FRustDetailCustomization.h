// MyCustomization.h
#pragma once

#include "IDetailCustomization.h"
#include "SRustDropdownList.h"

class FRustDetailCustomization: public IDetailCustomization
{
public:
	// IDetailCustomization interface
	virtual void CustomizeDetails(IDetailLayoutBuilder& DetailBuilder) override;
	//

	static TSharedRef< IDetailCustomization > MakeInstance();
};
