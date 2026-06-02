===description===
Continue is not break
===file===
<?php
                    switch(2) {
                        case 2:
                            echo "two
";
                            continue 2;
                    }
===expect===
ParseError@6:29-6:37: Parse error: Cannot 'continue' 2 levels
