# JNA — used by UniFFI-generated Kotlin bindings
-dontwarn java.lang.invoke.StringConcatFactory
-keep class com.sun.jna.** { *; }
-keepclassmembers class * extends com.sun.jna.** { public *; }
-keep class uniffi.aurafy_ffi.** { *; }
