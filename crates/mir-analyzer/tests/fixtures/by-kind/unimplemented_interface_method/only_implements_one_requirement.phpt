===description===
Only implements one requirement
===file===
<?php
use ImplementationRequirementsTraitImposesImplementationRequirements;
use ImplementationRequirementsBaseA;

class Invalid implements A {
    use ImposesImplementationRequirements;
}

===expect===
ParseError@2:4-2:68: Parse error: The use statement with non-compound name 'ImplementationRequirementsTraitImposesImplementationRequirements' has no effect
ParseError@3:4-3:35: Parse error: The use statement with non-compound name 'ImplementationRequirementsBaseA' has no effect
UndefinedClass@5:25-5:26: Class A does not exist
UndefinedTrait@6:8-6:41: Trait ImposesImplementationRequirements does not exist
