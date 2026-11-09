===description===
A trait method its using class replaces with its own isn't a parent of a subclass's override.
===file===
<?php
interface E {}
trait T { protected function m(E $e): array { return [$e]; } }
class B { use T { m as traitM; } protected function m(): array { return []; } }
class C extends B { protected function m(): array { return []; } }
