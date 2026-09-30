===description===
FirstClassCallable:UndefinedStaticMethod
===config===
suppress=MixedAssignment,UnusedVariable
===file===
<?php
class Widget {}
$closure = Widget::undefined(...);
//                 ^^^^^^^^^ UndefinedMethod: Method Widget::undefined() does not exist
$count = $closure();
===expect===
