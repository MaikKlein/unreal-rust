#include "K2Node_GetComponentRust.h"

#include "BlueprintActionDatabaseRegistrar.h"
#include "BlueprintNodeSpawner.h"
#include "EntityComponent.h"

#define LOCTEXT_NAMESPACE "K2Node_GetComponentRust"

void UK2Node_GetComponentRust::AllocateDefaultPins()
{
    Super::AllocateDefaultPins();
    CreatePin(EGPD_Input, UEdGraphSchema_K2::PC_Exec, UEdGraphSchema_K2::PN_Execute);
    //const auto ElementPin = CreatePin( EGPD_Input, UEdGraphSchema_K2::PC_Struct, FEntity::StaticStruct(),  TEXT("EntityId") );
    const auto ElementPin = CreatePin( EGPD_Input, UEdGraphSchema_K2::PC_Class, UObject::StaticClass(),  TEXT("EntityId") );
    const auto Out = CreatePin( EGPD_Output, UEdGraphSchema_K2::PC_Struct, TBaseStructure<FVector>::Get(),  TEXT("Velocity") );
	//UEdGraphPin* BlueprintPin = CreatePin(EGPD_Input, UEdGraphSchema_K2::PC_Object, UBlueprint::StaticClass(), FK2Node_SpawnActorHelper::BlueprintPinName);
}
void UK2Node_GetComponentRust::ExpandNode(class FKismetCompilerContext &CompilerContext, UEdGraph *SourceGraph)
{
}

void UK2Node_GetComponentRust::GetMenuActions(FBlueprintActionDatabaseRegistrar &ActionRegistrar) const
{
    const auto ActionKey = GetClass();
    if (ActionRegistrar.IsOpenForRegistration( ActionKey ))
    {
        const auto NodeSpawner = UBlueprintNodeSpawner::Create( ActionKey );
        check( NodeSpawner != nullptr );

        //auto Self = NewObject<UK2Node_GetComponentRust>();

        ActionRegistrar.AddBlueprintAction( ActionKey, NodeSpawner );
    }
}

FText UK2Node_GetComponentRust::GetNodeTitle(ENodeTitleType::Type TitleType) const
{
    return LOCTEXT("NodeTitle_NONE", "Get Component (Rust)");
}

#undef LOCTEXT_NAMESPACE