# JNA — used by UniFFI-generated Kotlin bindings
-dontwarn java.lang.invoke.StringConcatFactory
# java.awt is referenced by JNA's desktop code paths but does not exist on Android
-dontwarn java.awt.**
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { public *; }
-keep class uniffi.aurafy_ffi.** { *; }
