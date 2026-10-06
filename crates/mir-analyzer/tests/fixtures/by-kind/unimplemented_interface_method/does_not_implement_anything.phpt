===description===
Does not implement anything
===file===
<?php
use ImplementationRequirementsTraitImposesImplementationRequirements;
//  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ParseError: Parse error: The use statement with non-compound name 'ImplementationRequirementsTraitImposesImplementationRequirements' has no effect

class Invalid {
    use ImposesImplementationRequirements;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedTrait: Trait ImposesImplementationRequirements does not exist
}
