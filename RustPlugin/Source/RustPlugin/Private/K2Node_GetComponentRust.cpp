#include "K2Node_GetComponentRust.h"

#include "Api.h"
#include "BlueprintActionDatabaseRegistrar.h"
#include "BlueprintNodeSpawner.h"
#include "EntityComponent.h"
#include "K2Node_CallFunction.h"
#include "KismetCompiler.h"
#include "RustPlugin.h"
#include "SGraphNodeGetComponent.h"
#include "URustReflectionLibrary.h"

#define LOCTEXT_NAMESPACE "K2Node_GetComponentRust"

static const FName EntityParamName(TEXT("EntityId"));
static const FName UuidParamName(TEXT("Uuid"));

static const FName ReflectUuidParamName(TEXT("Id"));
static const FName ReflectOutputParamName(TEXT("Out"));
static const FName ReflectEntityParamName(TEXT("EntityId"));
static const FName ReflectIndexParamName(TEXT("Index"));

void UK2Node_GetComponentRust::AllocateDefaultPins()
{
	Super::AllocateDefaultPins();
	IndexPins.Empty();
	CreatePin(EGPD_Input, UEdGraphSchema_K2::PC_Exec, UEdGraphSchema_K2::PN_Execute);
	CreatePin(EGPD_Output, UEdGraphSchema_K2::PC_Exec, UEdGraphSchema_K2::PN_Then);
	const auto ElementPin = CreatePin(EGPD_Input, UEdGraphSchema_K2::PC_Struct, FEntity::StaticStruct(),
	                                  EntityParamName);

	const auto& Module = GetModule();


	FGuid Id = SelectedNode.Id;
	auto UuidPin = CreatePin(EGPD_Input, UEdGraphSchema_K2::PC_Object, UUuid::StaticClass(), UuidParamName);
	UuidPin->bHidden = true;
	auto UuidObject = NewObject<UUuid>();
	UuidObject->Id = ToUuid(Id);
	UuidPin->DefaultObject = UuidObject;

	uint32_t NumberOfFields = 0;
	Module.Plugin.Rust.reflection_fns.number_of_fields(ToUuid(Id), &NumberOfFields);
	UE_LOG(LogTemp, Warning, TEXT("Alloc %s, %i"), *SelectedNode.Name, NumberOfFields);

	for (uint32_t Idx = 0; Idx < NumberOfFields; Idx++)
	{
		const char* Name = nullptr;
		uintptr_t Len = 0;
		if (Module.Plugin.Rust.reflection_fns.get_field_name(ToUuid(Id), Idx, &Name, &Len))
		{
			FString IdxName = FString::FromInt(Idx);
			auto IdxPin = CreatePin(EGPD_Input, UEdGraphSchema_K2::PC_Int, *IdxName);
			IdxPin->DefaultValue = IdxName;

			ReflectionType Type = ReflectionType::Bool;
			if (Module.Plugin.Rust.reflection_fns.get_field_type(ToUuid(Id), Idx, &Type))
			{
				FString VarName = FString(Len, UTF8_TO_TCHAR(Name));
				if (Type == ReflectionType::Vector3)
				{
					CreatePin(EGPD_Output, UEdGraphSchema_K2::PC_Struct, TBaseStructure<FVector>::Get(),
					          *VarName);
				}
				if (Type == ReflectionType::Bool)
				{
					CreatePin(EGPD_Output, UEdGraphSchema_K2::PC_Boolean,
					          *VarName);
				}
				if (Type == ReflectionType::Float)
				{
					CreatePin(EGPD_Output, UEdGraphSchema_K2::PC_Float,
					          *VarName);
				}
			}
		}
	}
}

void UK2Node_GetComponentRust::ExpandNode(class FKismetCompilerContext& CompilerContext, UEdGraph* SourceGraph)
{
	Super::ExpandNode(CompilerContext, SourceGraph);

	Uuid Id = ToUuid(SelectedNode.Id);
	const auto& Module = GetModule();
	uint32_t NumberOfFields = 0;
	Module.Plugin.Rust.reflection_fns.number_of_fields(Id, &NumberOfFields);
	UE_LOG(LogTemp, Warning, TEXT("Expand %s, %i"), *SelectedNode.Name, NumberOfFields);
	UEdGraphPin* UUidPin = FindPinChecked(UuidParamName, EGPD_Input);
	UEdGraphPin* EntityPin = FindPinChecked(EntityParamName, EGPD_Input);
	UEdGraphPin* ThenPin = FindPinChecked(UEdGraphSchema_K2::PN_Then, EGPD_Output);

	UEdGraphPin* PrevExecPin = GetExecPin();
	for (uint32_t Idx = 0; Idx < NumberOfFields; Idx++)
	{
		const char* Name = nullptr;
		uintptr_t Len = 0;
		if (Module.Plugin.Rust.reflection_fns.get_field_name(Id, Idx, &Name, &Len))
		{
			FString IdxName = FString::FromInt(Idx);
			auto InputIdxPin = FindPinChecked(*IdxName);

			ReflectionType Type = ReflectionType::Bool;
			if (Module.Plugin.Rust.reflection_fns.get_field_type(Id, Idx, &Type))
			{
				UE_LOG(LogTemp, Warning, TEXT("Type"));
				FString VarName = FString(Len, UTF8_TO_TCHAR(Name));
				UEdGraphPin* OutputPin = FindPinChecked(*VarName, EGPD_Output);
				if (Type == ReflectionType::Vector3)
				{
					UE_LOG(LogTemp, Warning, TEXT("Call function"));
					UK2Node_CallFunction* CallFunctionNode = CompilerContext.SpawnIntermediateNode<
						UK2Node_CallFunction>(this, SourceGraph);
					CallFunctionNode->SetFromFunction(
						GetDefault<URustReflectionLibrary>()->FindFunctionChecked(
							GET_FUNCTION_NAME_CHECKED(URustReflectionLibrary, K2_GetReflectionVector3)));
					CallFunctionNode->AllocateDefaultPins();

					UEdGraphPin* CallExecPin = CallFunctionNode->GetExecPin();
					UEdGraphPin* CallThen = CallFunctionNode->GetThenPin();
					UEdGraphPin* CallUUid = CallFunctionNode->FindPinChecked(ReflectUuidParamName, EGPD_Input);
					UEdGraphPin* CallEntity = CallFunctionNode->FindPinChecked(ReflectEntityParamName, EGPD_Input);
					UEdGraphPin* CallIndex = CallFunctionNode->FindPinChecked(ReflectIndexParamName, EGPD_Input);
					UEdGraphPin* CallOut = CallFunctionNode->FindPinChecked(ReflectOutputParamName, EGPD_Output);

					//CompilerContext.MovePinLinksToIntermediate(*PrevExecPin, *CallExecPin);
					PrevExecPin->MakeLinkTo(CallExecPin);
					CompilerContext.MovePinLinksToIntermediate(*UUidPin, *CallUUid);
					CompilerContext.MovePinLinksToIntermediate(*EntityPin, *CallEntity);
					CompilerContext.MovePinLinksToIntermediate(*InputIdxPin, *CallIndex);
					CompilerContext.MovePinLinksToIntermediate(*CallOut, *OutputPin);

					PrevExecPin = CallThen;
				}
				if (Type == ReflectionType::Bool)
				{
				}
				if (Type == ReflectionType::Float)
				{
				}
			}
		}
	}
	//CompilerContext.MovePinLinksToIntermediate(*PrevExecPin, *ThenPin);
	PrevExecPin->MakeLinkTo(ThenPin);
	BreakAllNodeLinks();
}

void UK2Node_GetComponentRust::GetMenuActions(FBlueprintActionDatabaseRegistrar& ActionRegistrar) const
{
	const auto ActionKey = GetClass();
	if (ActionRegistrar.IsOpenForRegistration(ActionKey))
	{
		const auto NodeSpawner = UBlueprintNodeSpawner::Create(ActionKey);
		check(NodeSpawner != nullptr);

		//auto Self = NewObject<UK2Node_GetComponentRust>();

		ActionRegistrar.AddBlueprintAction(ActionKey, NodeSpawner);
	}
}

FText UK2Node_GetComponentRust::GetNodeTitle(ENodeTitleType::Type TitleType) const
{
	return LOCTEXT("NodeTitle_NONE", "Get Component (Rust)");
}

TSharedPtr<SGraphNode> UK2Node_GetComponentRust::CreateVisualWidget()
{
	//SNew(SGraphNodeGetComponent, this);
	return SNew(SGraphNodeGetComponent, this)
		.OnUuidPickedDelegate(
			FOnUuidPicked::CreateUObject(this, &UK2Node_GetComponentRust::OnUuidPicked));
}

void UK2Node_GetComponentRust::OnUuidPicked(FUuidViewNode* Node)
{
	UE_LOG(LogTemp, Warning, TEXT("Picked %s"), *Node->Name);
	SelectedNode = *Node;
	BreakAllOutputPins();
	ReconstructNode();
}

void UK2Node_GetComponentRust::BreakAllOutputPins()
{
	TSet<UEdGraphNode*> NodeList;

	NodeList.Add(this);

	// Iterate over each pin and break all links
	for (int32 PinIdx = 0; PinIdx < Pins.Num(); ++PinIdx)
	{
		UEdGraphPin* Pin = Pins[PinIdx];
		if (Pin->Direction != EEdGraphPinDirection::EGPD_Output)
			continue;

		// Save all the connected nodes to be notified below
		for (UEdGraphPin* Connection : Pin->LinkedTo)
		{
			NodeList.Add(Connection->GetOwningNode());
		}

		Pin->BreakAllPinLinks();
	}

	// Send a notification to all nodes that lost a connection
	for (UEdGraphNode* Node : NodeList)
	{
		Node->NodeConnectionListChanged();
	}
}

#undef LOCTEXT_NAMESPACE
