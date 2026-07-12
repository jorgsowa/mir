===description===
A class named only in an `@mixin` docblock tag must not be reported
UnusedClass.
===config===
suppress=
===file===
<?php
final class OnlyUsedViaMixin {}

/** @mixin OnlyUsedViaMixin */
class Consumer {}
===expect===
