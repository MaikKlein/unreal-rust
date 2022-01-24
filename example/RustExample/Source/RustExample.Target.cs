// Fill out your copyright notice in the Description page of Project Settings.

using UnrealBuildTool;
using System.Collections.Generic;

public class RustExampleTarget : TargetRules
{
	public RustExampleTarget(TargetInfo Target) : base(Target)
	{
		Type = TargetType.Game;
		DefaultBuildSettings = BuildSettingsVersion.V2;
		bUseUnityBuild = false;
		bUsePCHFiles = false;

		ExtraModuleNames.AddRange( new string[] { "RustExample" } );
	}
}
