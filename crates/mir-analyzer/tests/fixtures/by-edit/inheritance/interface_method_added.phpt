===description===
Adding an interface method reports the unchanged implementor.
===file:Shape.php===
<?php
interface Shape {}
===file:Square.php===
<?php
class Square implements Shape {}
===expect===
<<none>>
===edit:Shape.php===
<?php
interface Shape { public function area(): float; }
===expect===
Square.php: UnimplementedInterfaceMethod@2:0-2:32: Class Square must implement Shape::area() from interface
