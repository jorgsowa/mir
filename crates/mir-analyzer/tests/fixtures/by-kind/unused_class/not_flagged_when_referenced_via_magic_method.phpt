===description===
A class named only in a class-level `@method` docblock tag's return or
parameter type must not be reported UnusedClass.
===config===
suppress=
===file===
<?php
final class OnlyUsedInMagicMethodReturn {}
final class OnlyUsedInMagicMethodParam {}

/** @method OnlyUsedInMagicMethodReturn getThing(OnlyUsedInMagicMethodParam $arg) */
class Consumer {}
===expect===
