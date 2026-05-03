===description===
onlyImplementsOneRequirement
===file===
<?php
                    use ImplementationRequirementsTraitImposesImplementationRequirements;
                    use ImplementationRequirementsBaseA;

                    class Invalid implements A {
                        use ImposesImplementationRequirements;
                    }
                
===expect===
requires using class to implement
===ignore===
TODO
