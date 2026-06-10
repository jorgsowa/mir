===description===
Only implements one requirement
===ignore===
TODO
===file===
<?php
use ImplementationRequirementsTraitImposesImplementationRequirements;
use ImplementationRequirementsBaseA;

class Invalid implements A {
    use ImposesImplementationRequirements;
}

===expect===
ParseError@2:5-2:69: Parse error: The use statement with non-compound name 'ImplementationRequirementsTraitImposesImplementationRequirements' has no effect
ParseError@3:5-3:36: Parse error: The use statement with non-compound name 'ImplementationRequirementsBaseA' has no effect
UndefinedClass@5:26-5:27: Class A does not exist
UndefinedTrait@6:8-6:41: Trait ImposesImplementationRequirements does not exist
