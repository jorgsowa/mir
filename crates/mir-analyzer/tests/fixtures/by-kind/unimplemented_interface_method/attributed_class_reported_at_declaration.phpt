===description===
An attributed class is reported at its declaration, not at the attribute line.
===file===
<?php
interface Shape { public function area(): float; }
#[\AllowDynamicProperties]
final class Square implements Shape {}
//<^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UnimplementedInterfaceMethod: Class Square must implement Shape::area() from interface
