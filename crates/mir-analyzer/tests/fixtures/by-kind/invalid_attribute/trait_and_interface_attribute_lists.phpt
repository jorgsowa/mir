===description===
Attributes on a trait or interface are checked like those on a class.
===file===
<?php
final class Plain {}

#[Attribute(Attribute::TARGET_METHOD)]
final class OnMethod {}

#[Plain]
//^^^^^ InvalidAttribute: Class Plain does not have an #[Attribute] annotation
trait T {}

#[OnMethod]
//^^^^^^^^ InvalidAttribute: Attribute OnMethod cannot be used on this target
interface I {}

#[Missing]
//^^^^^^^ UndefinedAttributeClass: Attribute class Missing does not exist
interface J {}
