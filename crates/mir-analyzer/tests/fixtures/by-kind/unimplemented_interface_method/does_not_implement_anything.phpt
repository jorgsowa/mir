===description===
Does not implement anything
===file===
<?php
use ImplementationRequirementsTraitImposesImplementationRequirements;

class Invalid {
    use ImposesImplementationRequirements;
//      ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ UndefinedTrait: Trait ImposesImplementationRequirements does not exist
}
