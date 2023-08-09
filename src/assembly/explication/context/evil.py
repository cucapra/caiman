""" 
Ok, this is a super evil thing, but I'm too stupid to make macros work
So we're gonna do the _evil thing_ and just reflect code with Python

Uh, if we pretend this doesn't exist, we're all good, right?
"""

import re

def main():

    getters : list[str] = []

    with open("getters.rs", 'r') as f:
        current_function = ""
        open_count = 0
        started_function = False
        skip_next = False
        for line in f:
            if line.strip() == '// IMMUTABLE':
                skip_next = True
            if current_function or re.search(r'\s*pub\s+fn\s+get', line):
                current_function += line
                # allow for multiline arguments
                old_started = started_function
                started_function = line.count("{") > 0 or started_function
                if old_started != started_function:
                    print(f'copying function {line}')
                open_count += line.count("{") # completely safe
                open_count -= line.count("}") # completely legit
                if not open_count and started_function: # hmhmmm, very good
                    if skip_next:
                        skip_next = False
                    else:
                        getters.append(current_function)
                    current_function = ""
                    started_function = False
    
    print("Read getters succesfully")

    results = []
    for getter in getters:
        # first closing paren should always (?) be the end of the arguments
        arg_end = getter.find(")")
        arguments = getter[:arg_end]
        rest = getter[arg_end:]
        # rename the function with `_mut`
        arguments = re.sub(r'get(\S*?)(\s*?)\(', r'get\1_mut\2(', arguments)
        # &self is the only mutable argument
        arguments =  arguments.replace('&self', '&mut self')
        # every non-argument thing should be mutable!
        rest = rest.replace('&', '&mut ')
        # add _mut to all `get` calls
        rest = re.sub(r'get(\S*?)(\s*?)\(', r'get\1_mut\2(', rest)
        results.append(arguments + rest)

    print("Transformed results")

    mutators = []
    message = "// THIS AND THE FOLLOWING CODE IS GENERATED WITH evil.py, DO NOT TOUCH"
    with open("internal_mutators.rs", 'r') as f:
        for line in f:
            if line.strip() == message:
                break
            mutators.append(line)
    
    print("Read mutators file")

    with open("internal_mutators.rs", 'w') as f:
        for line in mutators:
            f.write(line)
        f.write(message + '\n\n')
        f.write('impl<\'context> Context<\'context> {\n')
        for result in results:
            f.write(result + '\n')
        f.write('}')

    print("Update finished")

if __name__ == "__main__":
    main()