from flask import Flask, jsonify, request, send_file
from tempfile import NamedTemporaryFile


class Product(object):
    def __init__(self, product_id: int, name: str, description: str):
        self.id = product_id
        self.name = name
        self.description = description
        self.icon_name = ""
        self.icon_file = None

    def to_dict(self):
        res = dict()
        res['id'] = self.id
        res['name'] = self.name
        res['description'] = self.description
        res['icon'] = self.icon_name
        return res


app = Flask(__name__)
products = dict()
last_id = 0


@app.route("/")
def hello_world():
    return "Hello"


@app.route("/product", methods=['POST'])
def add_product():
    global last_id
    curr_product = Product(last_id, request.args['name'], request.args['description'])
    products[last_id] = curr_product
    last_id += 1
    return jsonify(curr_product.to_dict()), 200


@app.route("/product/<product_id>", methods=['GET'])
def get_product(product_id):
    product_id = int(product_id)
    if product_id not in products:
        return "Not found", 404

    return jsonify(products[product_id].to_dict()), 200


@app.route("/product/<product_id>", methods=['PUT'])
def update_product(product_id):
    product_id = int(product_id)
    if product_id not in products:
        return "Not found", 404

    curr_product = products[product_id]
    if 'name' in request.args:
        curr_product.name = request.args['name']
    if 'description' in request.args:
        curr_product.description = request.args['description']
    if 'icon' in request.args:
        curr_product.icon_name = request.args['icon']
    return jsonify(curr_product.to_dict()), 200


@app.route("/product/<product_id>", methods=['DELETE'])
def delete_product(product_id):
    product_id = int(product_id)
    if product_id not in products:
        return "Not Found", 404

    curr_product = products.pop(product_id)
    return jsonify(curr_product.to_dict()), 200


@app.route("/products", methods=['GET'])
def get_all_products():
    return jsonify(list(map(lambda p: p.to_dict(), list(products.values()))))


@app.route("/product/<product_id>/image", methods=['POST'])
def upload_image(product_id):
    product_id = int(product_id)
    if product_id not in products:
        return "Not Found", 404

    curr_product = products[product_id]
    file = next(iter(request.files.values()))
    tmpfile = NamedTemporaryFile()
    file.save(tmpfile.name)
    curr_product.icon_name = file.filename
    curr_product.icon_file = tmpfile
    return "Uploaded", 200


@app.route("/product/<product_id>/image", methods=['GET'])
def get_image(product_id):
    product_id = int(product_id)
    if product_id not in products:
        return "Not Found", 404

    curr_product = products[product_id]
    if curr_product.icon_file is None:
        return "Not Found", 404

    return send_file(curr_product.icon_file, download_name=curr_product.icon_name)


if __name__ == "__main__":
    app.run(debug=False)
