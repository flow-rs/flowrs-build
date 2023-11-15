import {type DataflowNode} from "rete-engine";
import {ClassicPreset as Classic} from 'rete';
import {
    type ConstructorDefinition,
    type ConstructorDescription,
    type TypeDefinition,
    type TypeDescription
} from "~/repository/modules/packages";

const socket = new Classic.Socket('socket');

export class FlowrsNode extends Classic.Node implements DataflowNode {
    width = 500;
    height = 140;
    node_data: string | undefined;
    constructor_type: string = "New";

    constructor(name: string, typeDefinition: TypeDefinition, data: { [key: string]: any } | null, constructor_type: string) {
        super(name);

        this.constructor_type = constructor_type;
        this.setNodeData(data);
        this.addChooseConstructorAndControlInputs(typeDefinition.constructors)
        this.addInputs(typeDefinition);
        this.addOutputs(typeDefinition);
    }

    private addChooseConstructorAndControlInputs(constructors: ConstructorDefinition) {
        let currentConstructor;
        if (this.constructor_type == "New") {
            currentConstructor = constructors.New;
        } else if (this.constructor_type == "NewWithToken") {
            currentConstructor = constructors.NewWithToken;
        }
        if (!currentConstructor) {
            return;
        }
        let recordKeys = Object.keys(currentConstructor);
        if (recordKeys.length > 1) {
            console.error("This isn't supposed to be like that!", currentConstructor);
        }
        this.addControlInputs(currentConstructor[recordKeys[0]]);
    }

    private addControlInputs(constructor: ConstructorDescription) {
        if (!constructor.arguments) {
            return
        }
        for (const argument of constructor.arguments) {
            if (argument.construction.Constructor == "Json") {
                this.addControl(
                    'data',
                    new Classic.InputControl('text', {
                        initial: this.node_data, readonly: false, change: value => {
                            this.node_data = value;
                        }
                    })
                );
            }
        }

    }

    private setNodeData(data: { [key: string]: any } | null) {
        if (!data) {
            return
        }

        this.node_data = JSON.stringify(data, null, 2);
    }

    private addOutputs(types: TypeDefinition) {
        for (let name in types.outputs) {
            let typeDescription = types.outputs[name].type;
            let type_name = this.constructLabel(name, typeDescription);
            this.addOutput(name, new Classic.Output(socket, type_name));
        }
    }

    private addInputs(types: TypeDefinition) {
        for (let name in types.inputs) {
            let typeDescription = types.inputs[name].type;
            let type_name = this.constructLabel(name, typeDescription);
            this.addInput(name, new Classic.Input(socket, type_name));
            this.height += 20;
        }
    }

    private constructLabel(inputPairKey: string, typeDescription: TypeDescription) {
        let label = inputPairKey;
        if (typeDescription.Generic) {
            label += ' : ' + typeDescription.Generic.name;
        } else if (typeDescription.Type) {
            label += ' : ' + typeDescription.Type.name;
        } else {
            console.error("Type not supported", typeDescription)
        }
        return label;
    }

    data() {
        const value = this.node_data;
        return {value,};
    }
}