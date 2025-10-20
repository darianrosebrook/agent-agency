def calculateTotal(items):
    total=0
    for item in items:
        total=total+item.price
    return total

class userManager:
    def __init__(self,name):
        self.name=name
    def get_name(self):
        return self.name
