// Fill out your copyright notice in the Description page of Project Settings.


#include "AnimNotify_RustEvent.h"

#include "RustPlugin.h"
#include "RustUtils.h"
#include "Components/SkeletalMeshComponent.h"
#include "GameFramework/Actor.h"
#include "RustProperty.h"
#include "Containers/UnrealString.h"
#include "EntityComponent.h"

void UAnimNotify_RustEvent::Notify(USkeletalMeshComponent* MeshComp, UAnimSequenceBase* Animation,
                                   const FAnimNotifyEventReference& EventReference)
{
#if WITH_EDITORONLY_DATA
	UWorld* World = MeshComp->GetWorld();
	if (World && World->WorldType == EWorldType::EditorPreview)
	{
		// Don't send the event if we are previewing the animation
		return;
	}
#endif
	FGuid ParsedGuid;
	FGuid::Parse(Guid, ParsedGuid);


	auto Json = Event.SerializeToJson();
	auto ToUtf8 = FTCHARToUTF8(*Json);

	Utf8Str Utf8;
	Utf8.ptr = ToUtf8.Get();
	Utf8.len = ToUtf8.Length();
	if (MeshComp && MeshComp->GetOwner())
	{
		GetRustModule().Plugin.Rust.send_actor_event(static_cast<const AActorOpaque*>(MeshComp->GetOwner()),
		                                             ToUuid(ParsedGuid), Utf8);
	}
}
