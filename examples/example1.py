#! /usr/bin/env python

import json
import sys

def main():
    state = json.loads(sys.argv[1]).get('state', {})
    count = state.get('count', 0)

    state['count'] = count + 1
    print(state)


if __name__ == '__main__':
    main()
