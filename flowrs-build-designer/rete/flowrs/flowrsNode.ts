import {ClassicPreset, ClassicPreset as Classic} from 'rete';
import {
    type ConstructorDefinition,
    type ConstructorDescription,
    type TypeDefinition,
    type TypeDescription
} from "~/repository/modules/packages";
import {DropdownControl} from "~/rete/flowrs/editor";

const socket = new Classic.Socket('socket');

export class FlowrsNode extends Classic.Node<
    Record<string, ClassicPreset.Socket>,
    Record<string, ClassicPreset.Socket>,
    Record<
        string,
        | DropdownControl
        | ClassicPreset.InputControl<"number">
        | ClassicPreset.InputControl<"text">>> {
    width = 500;
    height = 140;
    public node_data: string | undefined;
    public genericsResolutionMap: Map<string, string> = new Map();
    private keyToTypeParameterMap: Map<string, string> = new Map();
    public constructor_type: string = "New";
    private compatibleTypes: Map<string, [string, TypeDefinition][]> = new Map();

    constructor(name: string,
                typeDefinition: TypeDefinition,
                data: { [key: string]: any } | null,
                constructor_type: string,
                typeParameters: { [key: string]: string } | null,
                allPossibleTypes: Map<string, TypeDefinition>
    ) {
        super(name);

        this.constructor_type = constructor_type;
        this.setNodeData(data);
        this.setTypeParameters(typeParameters);
        let constructorDescription = this.getConstructorDescription(typeDefinition.constructors);
        this.calculateCompatibleTypes(constructorDescription, allPossibleTypes);
        this.addControlInputs(constructorDescription);
        this.addDropdownControlsForGenericTypeParameters(typeDefinition.type_parameters);
        this.addInputs(typeDefinition);
        this.addOutputs(typeDefinition);
    }

    private getConstructorDescription(constructorDefinition: ConstructorDefinition) {
        let currentConstructor: Record<string, ConstructorDescription> | undefined;
        if (this.constructor_type == "New") {
            currentConstructor = constructorDefinition.New;
        } else if (this.constructor_type == "NewWithToken") {
            currentConstructor = constructorDefinition.NewWithToken;
        }
        if (!currentConstructor) {
            return;
        }
        let recordKeys = Object.keys(currentConstructor);
        if (recordKeys.length > 1) {
            console.error("Package endpoint malformed output?", currentConstructor);
        }
        return currentConstructor[recordKeys[0]];
    }

    private addControlInputs(constructor: ConstructorDescription | undefined) {
        if (!constructor?.arguments) {
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

    private addDropdownControlsForGenericTypeParameters(type_parameters: string[] | undefined) {
        if (!type_parameters) {
            return;
        }

        for (const typeParameter of type_parameters) {
            let possibleValues = this.compatibleTypes.get(typeParameter);
            if (!possibleValues) {
                continue
            }
            this.addControl(
                typeParameter,
                new DropdownControl(typeParameter, possibleValues, (selectedValue: [string, TypeDefinition]) => {
                    if (selectedValue?.at(0)) {
                        this.genericsResolutionMap.set(typeParameter, selectedValue[0]);
                        console.log("callback",selectedValue)
                    }
                })
            );
            this.height += 75;
        }
    }

    private setNodeData(data: { [key: string]: any } | null) {
        if (!data) {
            return
        }

        this.node_data = JSON.stringify(data, null, 2);
    }

    private setTypeParameters(typeParameters: { [p: string]: string } | null) {
        if (!typeParameters) {
            return;
        }

        for (const typeParametersKey in typeParameters) {
            this.genericsResolutionMap.set(typeParametersKey, typeParameters[typeParametersKey]);
        }

        console.log("Type parameters set:", this.genericsResolutionMap);
    }

    private calculateCompatibleTypes(constructorDescription: ConstructorDescription | undefined, allPossibleTypes: Map<string, TypeDefinition>) {
        if (!constructorDescription?.arguments) {
            return;
        }
        console.log("arguments", constructorDescription.arguments)
        for (const argument of constructorDescription.arguments) {
            console.log("check if fitting:", argument.type.Generic, argument.construction.Constructor)
            if (argument.type.Generic && argument.construction.Constructor) {
                this.compatibleTypes.set(argument.type.Generic.name, this.filterTypesWithConstructor(argument.construction.Constructor, allPossibleTypes))
            }
        }
    }

    private addOutputs(types: TypeDefinition) {
        for (let outputName in types.outputs) {
            let typeDescription = types.outputs[outputName].type;
            let typeName = this.getTypeName(typeDescription);
            this.addOutput(outputName, new Classic.Output(socket, outputName + ':' + typeName, false));
            this.keyToTypeParameterMap.set(outputName, typeName);
        }
    }

    private addInputs(types: TypeDefinition) {
        for (let inputName in types.inputs) {
            let typeDescription = types.inputs[inputName].type;
            let typeName = this.getTypeName(typeDescription);
            this.addInput(inputName, new Classic.Input(socket, inputName + ':' + typeName, false));
            this.keyToTypeParameterMap.set(inputName, typeName);
            this.height += 20;
        }
    }

    private getTypeName(typeDescription: TypeDescription) {
        if (typeDescription.Generic) {
            return typeDescription.Generic.name;
        } else if (typeDescription.Type) {
            return typeDescription.Type.name;
        } else {
            console.error("Type not supported", typeDescription)
        }
        return "unknown";
    }

    public getTypeForKey(key: string): string | undefined {
        let typeName = this.keyToTypeParameterMap.get(key);
        if (!typeName) {
            return;
        }
        let genericResolutionType = this.genericsResolutionMap.get(typeName);
        if (genericResolutionType) {
            return genericResolutionType
        }
        return typeName;
    }

    data() {
        const value = this.node_data;
        return {value,};
    }

    private filterTypesWithConstructor(constructorName: string, allPossibleTypes: Map<string, TypeDefinition>) {
        console.log("get types with constructor ", constructorName, allPossibleTypes)
        let filteredTypes = [];
        for (const possibleType of allPossibleTypes) {
            let typeDefinition = possibleType[1];
            let constructorDefinition = typeDefinition.constructors;
            console.log('possible type ', typeDefinition, constructorDefinition)
            for (const key in constructorDefinition.New) {
                console.log(key, constructorName)
                if (key == constructorName) {
                    filteredTypes.push(possibleType);
                }
            }
            for (const key in constructorDefinition.NewWithToken) {
                console.log(key, constructorName)
                if (key == constructorName) {
                    filteredTypes.push(possibleType);
                }
            }
        }
        console.log("filtered type:", filteredTypes)
        return filteredTypes;
    }
}