// Fill out your copyright notice in the Description page of Project Settings.

using UnrealBuildTool;
using System.Collections.Generic;

public class UnrealLearningKitGamesEditorTarget : TargetRules
{
	public UnrealLearningKitGamesEditorTarget(TargetInfo Target) : base(Target)
	{
		Type = TargetType.Editor;
		DefaultBuildSettings = BuildSettingsVersion.V2;
		bUseUnityBuild = false;
		ExtraModuleNames.AddRange( new string[] { "UnrealLearningKitGames" } );
	}
}
