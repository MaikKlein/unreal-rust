// Copyright Epic Games, Inc. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "Framework/Commands/Commands.h"
#include "RustPluginStyle.h"

class FRustPluginCommands : public TCommands<FRustPluginCommands>
{
public:

	FRustPluginCommands()
		: TCommands<FRustPluginCommands>(TEXT("RustPlugin"), NSLOCTEXT("Contexts", "RustPlugin", "RustPlugin Plugin"), NAME_None, FRustPluginStyle::GetStyleSetName())
	{
	}

	// TCommands<> interface
	virtual void RegisterCommands() override;

public:
	TSharedPtr< FUICommandInfo > OpenPluginWindow;
};