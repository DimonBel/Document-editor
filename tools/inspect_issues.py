import json, sys
d = json.load(sys.stdin)
refactor = [x for x in d if x['number'] >= 18]
open_ref = [x for x in refactor if x['state'] == 'OPEN']
closed_ref = [x for x in refactor if x['state'] == 'CLOSED']
print('Refactor issues (#18+):')
print('  OPEN:   ' + str(len(open_ref)))
for x in open_ref:
    print('    #%-3d %s' % (x['number'], x['title'][:70]))
print('  CLOSED: ' + str(len(closed_ref)))
