===description===
Does not implement anything
===file===
<?php
use ImplementationRequirementsTraitImposesImplementationRequirements;

class Invalid {
    use ImposesImplementationRequirements;
}

===expect===
ParseError@2:4-2:68: Parse error: The use statement with non-compound name 'ImplementationRequirementsTraitImposesImplementationRequirements' has no effect
UndefinedTrait@5:8-5:41: Trait ImposesImplementationRequirements does not exist
