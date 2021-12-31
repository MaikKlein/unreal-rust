#include "Api.h"

UnrealBindings CreateBindings() {
    UnrealBindings b;
    b.get_spatial_data = &GetSpatialData;
    b.set_spatial_data = &SetSpatialData;
    b.log = &Log;
    return b;
}

Quaternion ToQuaternion(FQuat q)
{
    Quaternion r;
    r.x = q.X;
    r.y = q.Y;
    r.z = q.Z;
    r.w = q.W;
    return r;
}

Vector3 ToVector3(FVector v)
{
    Vector3 r;
    r.x = v.X;
    r.y = v.Y;
    r.z = v.Z;
    return r;
}

FVector ToFVector(Vector3 v)
{
    return FVector(v.x, v.y, v.z);
}

// W, X, Y, Z
FQuat ToFQuat(Quaternion q)
{
    return FQuat(q.w, q.x, q.y, q.z);
}

AActor *ToAActor(const AActorOpaque *actor)
{
    return (AActor *)actor;
}
AActor *ToAActor(AActorOpaque *actor)
{
    return (AActor *)actor;
}